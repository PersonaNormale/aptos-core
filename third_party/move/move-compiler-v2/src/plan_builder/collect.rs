// Parts of the file are Copyright (c) The Diem Core Contributors
// Parts of the file are Copyright (c) The Move Contributors
// Parts of the file are Copyright (c) Aptos Foundation
// All Aptos Foundation code and content is licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

//! Groups and validates a function's test cases and their expected failures.

use super::{failure::parse_failure_attribute, test_plan::ExpectedFailure};
use codespan_reporting::diagnostic::Severity;
use legacy_move_compiler::shared::known_attributes::TestingAttribute;
use move_model::{
    ast::{Attribute, ModuleName},
    model::{AttributeSiblingId, FunctionEnv, GlobalEnv},
};
use std::collections::{BTreeMap, BTreeSet};

/// One `#[test(...)]` attribute of a (possibly parametric) test function, with its
/// `#[expected_failure]` already resolved.
pub(super) struct RawTestCase<'a> {
    pub(super) index: usize,
    pub(super) attr: &'a Attribute,
    pub(super) expected_failure: Option<ExpectedFailure>,
}

/// Attributes sharing one `#[...]` block.
#[derive(Default)]
struct TestAttribute<'a> {
    tests: Vec<&'a Attribute>,
    failures: Vec<&'a Attribute>,
    others: Vec<&'a Attribute>,
}

/// Result of one pass over a function's attributes: the `#[...]` blocks that contain a
/// `#[test]`, every `#[expected_failure]` in source order (attribute-local and standalone
/// combined), and the function's `#[test_only]` attribute, if any.
struct ClassifiedAttributes<'a> {
    test_groups: BTreeMap<AttributeSiblingId, TestAttribute<'a>>,
    all_failures: Vec<&'a Attribute>,
    test_only: Option<&'a Attribute>,
}

fn collect_test_attributes<'a>(
    env: &GlobalEnv,
    attrs: &'a [Attribute],
) -> ClassifiedAttributes<'a> {
    let test_name = env.symbol_pool().make(TestingAttribute::TEST);
    let ef_name = env.symbol_pool().make(TestingAttribute::EXPECTED_FAILURE);
    let test_only_name = env.symbol_pool().make(TestingAttribute::TEST_ONLY);

    let mut test_groups: BTreeMap<AttributeSiblingId, TestAttribute> = BTreeMap::new();
    let mut all_failures = Vec::new();
    let mut test_only = None;
    for attr in attrs {
        if attr.name() == ef_name {
            all_failures.push(attr);
        }
        if attr.name() == test_only_name {
            test_only = Some(attr);
        }
        let entry = test_groups.entry(attr.attribute_sibling_id()).or_default();
        if attr.name() == test_name {
            entry.tests.push(attr);
        } else if attr.name() == ef_name {
            entry.failures.push(attr);
        } else {
            entry.others.push(attr);
        }
    }
    test_groups.retain(|_, a| !a.tests.is_empty());
    ClassifiedAttributes {
        test_groups,
        all_failures,
        test_only,
    }
}

/// Collects test cases after validating their attribute groups.
pub(super) fn collect_and_validate_test_cases<'a>(
    env: &GlobalEnv,
    current_module: &ModuleName,
    function: &'a FunctionEnv,
) -> Vec<RawTestCase<'a>> {
    let attrs = function.get_attributes();
    let ClassifiedAttributes {
        test_groups,
        all_failures,
        test_only,
    } = collect_test_attributes(env, attrs);

    if test_groups.is_empty() {
        if let Some(abort_attribute) = all_failures.first() {
            let fn_id_loc = function.get_id_loc();
            let fn_msg = "only functions defined as a test with `#[test]` can also have an \
                          `#[expected_failure]` attribute";
            let abort_msg = "attributed as `#[expected_failure]` here";
            let abort_loc = env.get_node_loc(abort_attribute.node_id());
            env.error_with_labels(&fn_id_loc, fn_msg, vec![(abort_loc, abort_msg.to_string())]);
        }
        return Vec::new();
    }

    let single_case = test_groups.len() == 1;
    if validate_test_attributes(env, &test_groups, &all_failures, test_only, single_case) {
        return Vec::new();
    }

    let raw_cases = build_raw_test_cases(&test_groups, &all_failures, single_case);
    let raw_cases: Vec<RawTestCase> = raw_cases
        .into_iter()
        .map(|(index, attr, expected_failure_attr)| {
            let expected_failure = expected_failure_attr
                .and_then(|attr| parse_failure_attribute(env, current_module, attr).ok());
            RawTestCase {
                index,
                attr,
                expected_failure,
            }
        })
        .collect();

    check_zero_arg_distinctness(env, function, &raw_cases);
    raw_cases
}

fn validate_test_attributes(
    env: &GlobalEnv,
    attributes: &BTreeMap<AttributeSiblingId, TestAttribute>,
    all_failure_attrs: &[&Attribute],
    test_only_attr: Option<&Attribute>,
    single_case: bool,
) -> bool {
    let mut has_error = false;

    // test_only in a separate test attribute conflicts with #[test(...)].
    // If it shares an attribute with #[test], the unrelated-sibling check below catches it.
    if let Some(attr) = test_only_attr {
        if !attributes.contains_key(&attr.attribute_sibling_id()) {
            let test_only_loc = env.get_node_loc(attr.node_id());
            let first_test_attr = attributes
                .values()
                .next()
                .expect("test_groups is non-empty; caller returns early otherwise")
                .tests[0];
            let test_attribute_loc = env.get_node_loc(first_test_attr.node_id());
            env.diag_with_primary_and_labels(
                Severity::Error,
                &test_only_loc,
                "`#[test_only]` cannot be combined with `#[test(...)]` on the same function",
                "conflicts with the `#[test(...)]` annotation",
                vec![(test_attribute_loc, "previously annotated here".to_string())],
            );
            has_error = true;
        }
    }

    for attribute in attributes.values() {
        if attribute.tests.len() > 1 {
            let loc = env.get_node_loc(attribute.tests[1].node_id());
            env.diag_with_primary_and_labels(
                Severity::Error,
                &loc,
                "a test attribute may only contain one `#[test]`",
                "second `#[test]` here",
                vec![],
            );
            has_error = true;
        }
        for sibling in &attribute.others {
            let loc = env.get_node_loc(sibling.node_id());
            env.diag_with_primary_and_labels(
                Severity::Warning,
                &loc,
                "a test attribute may only contain `#[test]` and `#[expected_failure]`",
                "not allowed in a test attribute",
                vec![],
            );
        }
    }

    if single_case {
        if all_failure_attrs.len() > 1 {
            let loc = env.get_node_loc(all_failure_attrs[1].node_id());
            env.diag_with_primary_and_labels(
                Severity::Error,
                &loc,
                "a single case test function may only have one `#[expected_failure]`",
                "second occurrence here",
                vec![],
            );
            has_error = true;
        }
    } else {
        // A standalone expected failure has no case to apply to when there are multiple cases.
        for failure in all_failure_attrs {
            if !attributes.contains_key(&failure.attribute_sibling_id()) {
                let loc = env.get_node_loc(failure.node_id());
                env.diag_with_primary_notes_and_labels(
                    Severity::Warning,
                    &loc,
                    "`#[expected_failure]` on a parametric multi case test must belong to one of its test attributes",
                    "not part of any test attribute",
                    vec![
                        "move this attribute inside the test attribute it applies to"
                            .to_string(),
                    ],
                    vec![],
                );
            }
        }
        for attribute in attributes.values() {
            for extra in attribute.failures.iter().skip(1) {
                let loc = env.get_node_loc(extra.node_id());
                env.diag_with_primary_and_labels(
                    Severity::Warning,
                    &loc,
                    "a test attribute may only have one `#[expected_failure]`",
                    "extra occurrence here",
                    vec![],
                );
            }
        }
    }

    has_error
}

/// Builds one `(index, test attr, expected-failure attr)` triple per test group, in source
/// order. The expected-failure attr is resolved into a value by the caller.
fn build_raw_test_cases<'a>(
    attributes: &BTreeMap<AttributeSiblingId, TestAttribute<'a>>,
    all_failure_attrs: &[&'a Attribute],
    single_case: bool,
) -> Vec<(usize, &'a Attribute, Option<&'a Attribute>)> {
    // For a single case: the standalone top-level EF (if any) belongs to that one case.
    let standalone_failure: Option<&'a Attribute> = if single_case {
        all_failure_attrs
            .iter()
            .find(|a| !attributes.contains_key(&a.attribute_sibling_id()))
            .copied()
    } else {
        None
    };

    // BTreeMap iteration is in AttributeSiblingId order = source order.
    attributes
        .values()
        .enumerate()
        .map(|(index, attribute)| {
            // Extra expected failures have already been diagnosed; keep only the first.
            let expected_failure_attr = attribute.failures.first().copied().or(standalone_failure);
            (index, attribute.tests[0], expected_failure_attr)
        })
        .collect()
}

/// Warns when zero-argument cases repeat the same expected failure.
fn check_zero_arg_distinctness(env: &GlobalEnv, function: &FunctionEnv, raw_cases: &[RawTestCase]) {
    if raw_cases.len() <= 1 || !function.get_parameters_ref().is_empty() {
        return;
    }
    let distinct: BTreeSet<&Option<ExpectedFailure>> = raw_cases
        .iter()
        .map(|case| &case.expected_failure)
        .collect();
    if distinct.len() < raw_cases.len() {
        let loc = env.get_node_loc(raw_cases[0].attr.node_id());
        env.diag_with_primary_and_labels(
            Severity::Warning,
            &loc,
            "redundant parametric cases on a zero-argument function",
            "at least one case must carry a differentiating `#[expected_failure]`",
            vec![],
        );
    }
}
