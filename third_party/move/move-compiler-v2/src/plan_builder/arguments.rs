// Parts of the file are Copyright (c) The Diem Core Contributors
// Parts of the file are Copyright (c) The Move Contributors
// Parts of the file are Copyright (c) Aptos Foundation
// All Aptos Foundation code and content is licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

//! Builds the concrete `MoveValue` argument list a `#[test(...)]` attribute assigns to a
//! function's parameters.

use super::{
    collect::RawTestCase,
    convert::ToMoveValue,
    error::{Checked, ErrorReported},
};
use codespan_reporting::diagnostic::Severity;
use legacy_move_compiler::shared::known_attributes::TestingAttribute;
use move_core_types::value::MoveValue;
use move_model::{
    ast::Attribute,
    model::{FunctionEnv, GlobalEnv, Parameter},
    symbol::Symbol,
    ty::{PrimitiveType, Type},
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
    let param_names: BTreeSet<Symbol> = parameters
        .iter()
        .map(|Parameter(var, _, _)| *var)
        .collect();

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
            Some(MoveValue::Address(addr)) => match ty {
                Type::Primitive(PrimitiveType::Signer) => arguments.push(MoveValue::Signer(*addr)),
                Type::Reference(_, inner) if **inner == Type::Primitive(PrimitiveType::Signer) => {
                    arguments.push(MoveValue::Signer(*addr));
                },
                Type::Primitive(PrimitiveType::Address) => {
                    arguments.push(MoveValue::Address(*addr))
                },
                _ => {
                    env.diag_with_primary_and_labels(
                        Severity::Error,
                        &test_attribute_loc,
                        "unable to generate test: unexpected argument type",
                        "expected an `address` or `signer`",
                        vec![(
                            var_loc.clone(),
                            "corresponding to this parameter".to_string(),
                        )],
                    );
                },
            },
            Some(value) => arguments.push(value.clone()),
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

/// Recursively flattens a `#[test(...)]` attribute tree into a `param name -> value` map.
///
/// A repeated parameter assignment warns; the first assignment is retained and subsequent ones
/// for the same name are ignored.
fn parse_test_attribute(
    env: &GlobalEnv,
    test_attribute: &Attribute,
    depth: usize,
) -> Checked<BTreeMap<Symbol, MoveValue>> {
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
            let move_value = value.to_move_value(env).map_err(|ErrorReported| {
                let loc = env.get_node_loc(*node_id);
                env.error_with_labels(&loc, "unsupported attribute value", vec![(
                    loc.clone(),
                    "assigned in this attribute".to_string(),
                )]);
                ErrorReported
            })?;
            let mut args = BTreeMap::new();
            args.insert(*name, move_value);
            Ok(args)
        },
    }
}
