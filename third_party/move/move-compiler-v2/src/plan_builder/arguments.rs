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
    ast::{Attribute, AttributeValue},
    model::{FunctionEnv, GlobalEnv, Loc, Parameter},
    symbol::Symbol,
    ty::{PrimitiveType, Type, TypeDisplayContext},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn build_case_arguments(
    env: &GlobalEnv,
    raw_case: &RawTestCase,
    function: &FunctionEnv,
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
                Some(target) => match to_move_value(value, &target, env) {
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
/// type, or `None` if `ty` is not a supported parameter type (a struct or other non-primitive,
/// non-vector type). Recurses through `Type::Vector` so `vector<vector<u8>>` is supported to
/// unbounded depth, the same as any other `vector<T>`.
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
                AttributeValue::Value(..) | AttributeValue::Vector(..) => {
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
