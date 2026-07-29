// Parts of the file are Copyright (c) The Diem Core Contributors
// Parts of the file are Copyright (c) The Move Contributors
// Parts of the file are Copyright (c) Aptos Foundation
// All Aptos Foundation code and content is licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

//! Builds the concrete `MoveValue` argument list a `#[test(...)]` attribute assigns to a
//! function's parameters.

use super::{
    collect::RawTestCase,
    convert::{to_move_value, ConversionError},
    error::{Checked, ErrorReported},
};
use codespan_reporting::diagnostic::Severity;
use legacy_move_compiler::shared::known_attributes::TestingAttribute;
use move_core_types::value::MoveValue;
use move_model::{
    ast::{Attribute, AttributeValue, ModuleName},
    model::{FunctionEnv, GlobalEnv, Loc, Parameter},
    symbol::Symbol,
    ty::{PrimitiveType, Type, TypeDisplayContext},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn build_case_arguments(
    env: &GlobalEnv,
    raw_case: &RawTestCase,
    function: &FunctionEnv,
    current_module: &ModuleName,
) -> Vec<MoveValue> {
    let test_attribute = raw_case.attr;
    let test_attribute_loc = env.get_node_loc(test_attribute.node_id());
    let Ok(test_annotation_params) = parse_test_attribute(env, test_attribute, 0) else {
        return Vec::new();
    };

    let parameters = function.get_parameters_ref();
    let param_names: BTreeSet<Symbol> =
        parameters.iter().map(|Parameter(var, _, _)| *var).collect();

    // Check for unknown assignments (names not in the function parameter list).
    if let Attribute::Apply {
        attrs: inner_attrs, ..
    } = test_attribute
    {
        for inner in inner_attrs {
            if let Attribute::Assign { name, node_id, .. } = inner {
                if !param_names.contains(name) {
                    let loc = env.get_node_loc(*node_id);
                    env.diag_with_primary_and_labels(
                        Severity::Warning,
                        &loc,
                        "unknown test parameter assignment",
                        &format!("no parameter named `{}`", env.symbol_pool().string(*name)),
                        vec![],
                    );
                }
            }
        }
    }

    let mut arguments = Vec::new();
    for param in parameters {
        let Parameter(var, ty, var_loc) = &param;

        match test_annotation_params.get(var) {
            Some(value) => match supported_param_type(ty) {
                Some(target) => match to_move_value(value, &target, current_module, env) {
                    Ok(move_value) => arguments.push(move_value),
                    Err(err) => report_conversion_error(env, &test_attribute_loc, var_loc, err),
                },
                None => report_conversion_error(
                    env,
                    &test_attribute_loc,
                    var_loc,
                    ConversionError::UnsupportedParameterType,
                ),
            },
            None => {
                env.diag_with_primary_and_labels(
                    Severity::Error,
                    &test_attribute_loc,
                    "unable to generate test: missing parameter assignment",
                    "expected a parameter to be assigned in this attribute",
                    vec![(
                        var_loc.clone(),
                        "corresponding to this parameter".to_string(),
                    )],
                );
            },
        }
    }
    arguments
}

/// The `Type` a `#[test(...)]` assignment must be checked against for this declared parameter
/// type, or `None` if `ty` is not a supported parameter type. Recurses through `Type::Vector` and
/// `Type::Struct`'s type arguments so `vector<vector<u8>>` and `Wrapper<Wrapper<u8>>` are both
/// supported to unbounded depth, the same as any other `vector<T>`/generic struct. Struct field
/// types are not checked here (that needs `env`, which this function doesn't take); unsupported
/// field types are rejected later, inside `to_move_value`'s `Pack` arm, per field.
///
/// `&signer` is the only reference type accepted, matching the one special case Move's own test
/// harness constructs by reference. No other primitive is accepted by reference: `ty` being e.g.
/// `&u8` is not a supported parameter type either, and neither is `vector<T>` behind a reference.
fn supported_param_type(ty: &Type) -> Option<Type> {
    match ty {
        Type::Primitive(p) => Some(Type::Primitive(*p)),
        Type::Reference(_, inner) if **inner == Type::Primitive(PrimitiveType::Signer) => {
            Some(Type::Primitive(PrimitiveType::Signer))
        },
        Type::Vector(inner) => supported_param_type(inner).map(|t| Type::Vector(Box::new(t))),
        Type::Struct(mid, sid, args) => {
            let all_args_supported = args.iter().all(|a| supported_param_type(a).is_some());
            all_args_supported.then(|| Type::Struct(*mid, *sid, args.clone()))
        },
        _ => None,
    }
}

/// Reports a `ConversionError` from `to_move_value` at the attribute's own location, labeling
/// the specific parameter it was assigned to - the two-location pattern `build_case_arguments`
/// already used for its "expected an address or signer" diagnostic before this layer.
fn report_conversion_error(
    env: &GlobalEnv,
    test_attribute_loc: &Loc,
    var_loc: &Loc,
    err: ConversionError,
) {
    // `InvalidUtf8` is the one error that points at its own argument node rather than at the
    // whole attribute, so it is handled separately instead of threading a per-arm location
    // through every other case below.
    if let ConversionError::InvalidUtf8 { node_id } = err {
        let loc = env.get_node_loc(node_id);
        env.diag_with_primary_and_labels(
            Severity::Error,
            &loc,
            "unable to generate test: invalid UTF-8",
            "this byte sequence is not valid UTF-8",
            vec![(
                var_loc.clone(),
                "corresponding to this parameter".to_string(),
            )],
        );
        return;
    }

    let (msg, note) = match err {
        ConversionError::NotANumber => (
            "unable to generate test: unexpected argument type",
            "expected a numeric literal".to_string(),
        ),
        ConversionError::NotAnAddress => (
            "unable to generate test: unexpected argument type",
            "expected an `address` or `signer` literal".to_string(),
        ),
        ConversionError::NotABool => (
            "unable to generate test: unexpected argument type",
            "expected a `bool` literal (`true` or `false`)".to_string(),
        ),
        ConversionError::TypeMismatch { declared } => {
            let ctx = TypeDisplayContext::new(env);
            (
                "unable to generate test: mismatched types",
                format!(
                    "attribute value is explicitly typed `{}`, which disagrees with this \
                     parameter",
                    declared.display(&ctx)
                ),
            )
        },
        ConversionError::OutOfRange { min, max } => (
            "unable to generate test: literal out of range",
            format!(
                "value must be between `{}` and `{}` for this parameter",
                min, max
            ),
        ),
        ConversionError::UnsupportedParameterType => (
            "unable to generate test: unsupported parameter type",
            "test attribute assignments only support `signer`, `address`, `bool`, and integer \
             parameters"
                .to_string(),
        ),
        ConversionError::UnknownStruct => (
            "unable to generate test: unsupported parameter type",
            "no struct with this name was found".to_string(),
        ),
        ConversionError::VariantOnNonEnum { struct_id, variant } => {
            let struct_env = env.get_struct(struct_id);
            (
                "unable to generate test: not an enum",
                format!(
                    "`{}` is not a variant: `{}` has no variants",
                    variant.display(env.symbol_pool()),
                    struct_env.get_full_name_str()
                ),
            )
        },
        ConversionError::VariantRequired { struct_id } => {
            let struct_env = env.get_struct(struct_id);
            let example = struct_env
                .get_variants()
                .next()
                .map(|v| v.display(env.symbol_pool()).to_string())
                .unwrap_or_default();
            (
                "unable to generate test: missing variant",
                format!(
                    "`{}` is an enum; a variant must be selected, e.g. `{}::{}`",
                    struct_env.get_full_name_str(),
                    struct_env.get_full_name_str(),
                    example
                ),
            )
        },
        ConversionError::UnknownVariant { struct_id, variant } => {
            let struct_env = env.get_struct(struct_id);
            let known = struct_env
                .get_variants()
                .map(|v| v.display(env.symbol_pool()).to_string())
                .collect::<Vec<_>>()
                .join(", ");
            (
                "unable to generate test: unknown variant",
                format!(
                    "no variant named `{}` on enum `{}`; known variants: {}",
                    variant.display(env.symbol_pool()),
                    struct_env.get_full_name_str(),
                    known
                ),
            )
        },
        ConversionError::StructNotConstructible { struct_id } => {
            let struct_env = env.get_struct(struct_id);
            (
                "unable to generate test: struct not constructible here",
                format!(
                    "`{}` can only be constructed within module `{}`",
                    struct_env.get_full_name_str(),
                    struct_env.module_env.get_full_name_str()
                ),
            )
        },
        ConversionError::ConstructorMismatch {
            expected_positional,
        } => (
            "unable to generate test: wrong constructor kind",
            if expected_positional {
                "expected a positional constructor `Name(..)`".to_string()
            } else {
                "expected a struct constructor `Name { .. }`".to_string()
            },
        ),
        ConversionError::MissingFields(names) => (
            "unable to generate test: missing struct fields",
            format!(
                "missing field(s): {}",
                names
                    .iter()
                    .map(|s| s.display(env.symbol_pool()).to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ),
        ConversionError::UnknownField(name) => (
            "unable to generate test: unknown struct field",
            format!(
                "no field named `{}` on this struct",
                name.display(env.symbol_pool())
            ),
        ),
        ConversionError::FieldCountMismatch { expected, found } => (
            "unable to generate test: wrong number of positional fields",
            format!("expected {} field(s), found {}", expected, found),
        ),
        ConversionError::InvalidUtf8 { .. } => {
            unreachable!("InvalidUtf8 is handled by the early return above")
        },
    };
    env.diag_with_primary_and_labels(Severity::Error, test_attribute_loc, msg, &note, vec![(
        var_loc.clone(),
        "corresponding to this parameter".to_string(),
    )]);
}

/// Recursively flattens a `#[test(...)]` attribute tree into a `param name -> value` map.
///
/// A repeated parameter assignment warns; the first assignment is retained and subsequent ones
/// for the same name are ignored.
fn parse_test_attribute(
    env: &GlobalEnv,
    test_attribute: &Attribute,
    depth: usize,
) -> Checked<BTreeMap<Symbol, AttributeValue>> {
    match test_attribute {
        Attribute::Apply { node_id, .. } if depth > 0 => {
            let loc = env.get_node_loc(*node_id);
            env.error(&loc, "unexpected nested attribute in test declaration");
            Err(ErrorReported)
        },
        Attribute::Apply {
            name,
            attrs: params,
            ..
        } => {
            assert!(
                *TestingAttribute::TEST == env.symbol_pool().string(*name).to_string(),
                "ICE: We should only be parsing a raw test attribute"
            );
            let mut seen: BTreeSet<Symbol> = BTreeSet::new();
            for inner in params {
                if let Attribute::Assign { name, node_id, .. } = inner {
                    if !seen.insert(*name) {
                        let loc = env.get_node_loc(*node_id);
                        env.diag_with_primary_and_labels(
                            Severity::Warning,
                            &loc,
                            "a test parameter may only be assigned once",
                            "extra occurrence here",
                            vec![],
                        );
                    }
                }
            }
            let mut combined = BTreeMap::new();
            for attr in params {
                let partial = parse_test_attribute(env, attr, depth + 1)?;
                for (name, value) in partial {
                    combined.entry(name).or_insert(value);
                }
            }
            Ok(combined)
        },
        Attribute::Assign {
            node_id,
            name,
            value,
            ..
        } => {
            if depth != 1 {
                let loc = env.get_node_loc(*node_id);
                env.error(&loc, "unexpected nested attribute in test declaration");
                return Err(ErrorReported);
            }
            match value {
                AttributeValue::Value(..)
                | AttributeValue::Vector(..)
                | AttributeValue::Pack(..) => {
                    let mut args = BTreeMap::new();
                    args.insert(*name, value.clone());
                    Ok(args)
                },
                AttributeValue::Name(..) => {
                    let loc = env.get_node_loc(*node_id);
                    env.error_with_labels(&loc, "unsupported attribute value", vec![(
                        loc.clone(),
                        "assigned in this attribute".to_string(),
                    )]);
                    Err(ErrorReported)
                },
            }
        },
    }
}
