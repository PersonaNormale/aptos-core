// Parts of the file are Copyright (c) The Diem Core Contributors
// Parts of the file are Copyright (c) The Move Contributors
// Parts of the file are Copyright (c) Aptos Foundation
// All Aptos Foundation code and content is licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

//! Builds the named `TestCase`s for one function.

use super::{arguments::build_case_arguments, collect::collect_and_validate_test_cases};
use legacy_move_compiler::unit_test::TestCase;
use move_model::{
    ast::ModuleName,
    model::{FunctionEnv, GlobalEnv},
};

pub(super) fn build_test_info(
    env: &GlobalEnv,
    current_module: &ModuleName,
    function: FunctionEnv,
) -> Vec<(String, TestCase)> {
    let fn_name_str = function.get_name_str();
    let raw_cases = collect_and_validate_test_cases(env, current_module, &function);

    if raw_cases.len() == 1 {
        let raw_case = raw_cases
            .into_iter()
            .next()
            .expect("raw_cases.len() == 1 checked above");
        let arguments = build_case_arguments(env, &raw_case, &function);
        let test_case = TestCase {
            function_name: fn_name_str.clone(),
            arguments,
            expected_failure: raw_case.expected_failure,
        };
        return vec![(fn_name_str, test_case)];
    }

    raw_cases
        .into_iter()
        .map(|raw_case| {
            let arguments = build_case_arguments(env, &raw_case, &function);
            let case_name = format!("{}@case{}", fn_name_str, raw_case.index);
            let test_case = TestCase {
                function_name: fn_name_str.to_string(),
                arguments,
                expected_failure: raw_case.expected_failure,
            };
            (case_name, test_case)
        })
        .collect()
}
