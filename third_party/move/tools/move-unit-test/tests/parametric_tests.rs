// Copyright (c) Aptos Foundation
// Licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

use move_core_types::{account_address::AccountAddress, value::MoveValue};
use move_unit_test::{test_reporter::UnitTestFactoryWithCostTable, UnitTestingConfig};
use std::fs;
use tempfile::tempdir;

const TWO_CASE_SOURCE: &str = r#"
    address 0x1 {
    module M {
        #[test(addr = @0x1)]
        #[test(addr = @0x2)]
        fun foo(addr: signer) {
            let _ = addr;
        }
    }
    }
"#;

fn build_test_plan_from_source(source: &str) -> move_compiler_v2::plan_builder::TestPlan {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("case_identity.move");
    fs::write(&source_path, source).unwrap();

    let mut config = UnitTestingConfig::default()
        .with_named_addresses(move_stdlib::move_stdlib_named_addresses());
    config.source_files = vec![source_path.to_string_lossy().into_owned()];
    config.dep_files = move_stdlib::move_stdlib_files();
    config.build_test_plan().unwrap()
}

fn run_source(source: &str, filter: Option<&str>, report_statistics: bool) -> String {
    let plan = build_test_plan_from_source(source);
    let config = UnitTestingConfig {
        filter: filter.map(str::to_string),
        num_threads: 1,
        report_statistics,
        ..UnitTestingConfig::default()
    };
    let (output, ok) = config
        .run_and_report_unit_tests(
            plan,
            None,
            None,
            Vec::new(),
            UnitTestFactoryWithCostTable::new(None, None),
            false,
            false,
        )
        .unwrap();
    assert!(ok);
    String::from_utf8(output).unwrap()
}

#[test]
fn parametric_cases_separate_case_and_function_identity_in_source_order() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(addr = @0x0)]
            #[test(addr = @0x1)]
            #[test(addr = @0x2)]
            fun ordered(addr: signer) {
                let _ = addr;
            }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.len(), 3);
    for index in 0..3 {
        let test_case = module.tests.get(&format!("ordered@case{index}")).unwrap();
        assert_eq!(test_case.function_name, "ordered");
        assert_eq!(test_case.arguments, vec![MoveValue::Signer(
            AccountAddress::from_hex_literal(&format!("0x{index:x}")).unwrap()
        )]);
    }
}

#[test]
fn single_case_keeps_unsuffixed_case_and_function_identity() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(addr = @0x1)]
            fun single(addr: signer) {
                let _ = addr;
            }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();
    let test_case = module.tests.get("single").unwrap();

    assert_eq!(module.tests.len(), 1);
    assert_eq!(test_case.function_name, "single");
}

#[test]
fn parametric_case_filter_selects_one_case() {
    let output = run_source(TWO_CASE_SOURCE, Some("foo@case1"), false);
    assert!(output.contains("::foo@case1"));
    assert!(!output.contains("::foo@case0"));
    assert!(output.contains("Total tests: 1; passed: 1; failed: 0"));
}

#[test]
fn plain_function_name_filter_selects_all_cases() {
    let output = run_source(TWO_CASE_SOURCE, Some("foo"), false);
    assert!(output.contains("::foo@case0"));
    assert!(output.contains("::foo@case1"));
    assert!(output.contains("Total tests: 2; passed: 2; failed: 0"));
}

#[test]
fn partial_case_suffix_filter_matches_nothing() {
    let output = run_source(TWO_CASE_SOURCE, Some("foo@case"), false);
    assert!(!output.contains("::foo@case0"));
    assert!(!output.contains("::foo@case1"));
    assert!(output.contains("Total tests: 0"));
}

#[test]
fn parametric_statistics_use_case_identity() {
    let output = run_source(TWO_CASE_SOURCE, None, true);
    let statistics = output.split("Test Statistics:").nth(1).unwrap();

    assert!(statistics.contains("::foo@case0"));
    assert!(statistics.contains("::foo@case1"));
    assert_eq!(statistics.matches("::foo@case").count(), 2);
}

#[test]
fn vector_parameter_test_case_actually_executes() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(xs = vector[1, 2, 3])]
            fun test_vector(xs: vector<u8>) {
                assert!(std::vector::length(&xs) == 3, 0);
                assert!(*std::vector::borrow(&xs, 0) == 1, 1);
                assert!(*std::vector::borrow(&xs, 1) == 2, 2);
                assert!(*std::vector::borrow(&xs, 2) == 3, 3);
            }
        }
        }
    "#;

    let output = run_source(source, None, false);
    assert!(output.contains("Total tests: 1; passed: 1; failed: 0"));
}

#[test]
fn struct_parameter_test_case_actually_executes() {
    let source = r#"
        address 0x1 {
        module M {
            struct Point has copy, drop { x: u8, y: u8 }

            #[test(p = Point { x: 1, y: 2 })]
            fun test_point(p: Point) {
                assert!(p.x == 1, 0);
                assert!(p.y == 2, 1);
            }
        }
        }
    "#;

    let output = run_source(source, None, false);
    assert!(output.contains("Total tests: 1; passed: 1; failed: 0"));
}

#[test]
fn phantom_struct_parameter_test_case_actually_executes() {
    let source = r#"
        address 0x1 {
        module M {
            struct Phantom<phantom T> has copy, drop { val: u8 }

            #[test(p = Phantom<u8> { val: 5 })]
            fun test_phantom(p: Phantom<u8>) {
                assert!(p.val == 5, 0);
            }
        }
        }
    "#;

    let output = run_source(source, None, false);
    assert!(output.contains("Total tests: 1; passed: 1; failed: 0"));
}

#[test]
fn enum_variant_parameter_test_case_actually_executes() {
    let source = r#"
        address 0x1 {
        module M {
            enum Either has copy, drop { Left(u8), Right(u8) }

            #[test(e = Either::Left(5))]
            fun test_either(e: Either) {
                match (e) {
                    Either::Left(x) => assert!(x == 5, 0),
                    Either::Right(_) => assert!(false, 1),
                }
            }
        }
        }
    "#;

    let output = run_source(source, None, false);
    assert!(output.contains("Total tests: 1; passed: 1; failed: 0"));
}

#[test]
fn same_module_named_constant_actually_executes() {
    let source = r#"
        address 0x1 {
        module M {
            const BIG: u64 = 1000;

            #[test(x = BIG)]
            fun t(x: u64) {
                assert!(x == 1000, 0);
            }
        }
        }
    "#;

    let output = run_source(source, None, false);
    assert!(output.contains("Total tests: 1; passed: 1; failed: 0"));
}

#[test]
fn builtin_range_constant_actually_executes() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(x = MAX_U8)]
            fun u8_max(x: u8) {
                assert!(x == 255, 0);
            }

            #[test(x = MIN_I64)]
            fun i64_min(x: i64) {
                assert!(x == -9223372036854775808i64, 0);
            }
        }
        }
    "#;

    let output = run_source(source, None, false);
    assert!(output.contains("Total tests: 2; passed: 2; failed: 0"));
}

#[test]
fn cross_module_public_named_constant_actually_executes() {
    let source = r#"
        address 0x1 {
        module consts {
            public const LIMIT: u32 = 42;
        }
        module m {
            use 0x1::consts;

            #[test(x = consts::LIMIT)]
            fun t(x: u32) {
                assert!(x == 42, 0);
            }
        }
        }
    "#;

    let output = run_source(source, None, false);
    assert!(output.contains("Total tests: 1; passed: 1; failed: 0"));
}

#[test]
fn cross_module_private_named_constant_actually_executes() {
    let source = r#"
        address 0x1 {
        module consts {
            const EINVALID_STATE: u64 = 1;
        }
        module m {
            #[test(x = 0x1::consts::EINVALID_STATE)]
            fun t(x: u64) {
                assert!(x == 1, 0);
            }
        }
        }
    "#;

    let output = run_source(source, None, false);
    assert!(output.contains("Total tests: 1; passed: 1; failed: 0"));
}

#[test]
fn shadowing_local_constant_wins_over_builtin_actually_executes() {
    let source = r#"
        address 0x1 {
        module m {
            const MAX_U8: bool = false;

            #[test(x = MAX_U8)]
            fun t(x: bool) {
                assert!(!x, 0);
            }
        }
        }
    "#;

    let output = run_source(source, None, false);
    assert!(output.contains("Total tests: 1; passed: 1; failed: 0"));
}

#[test]
fn nested_in_vector_named_constant_actually_executes() {
    let source = r#"
        address 0x1 {
        module m {
            const A: u8 = 1;

            #[test(xs = vector[A, MAX_U8])]
            fun t(xs: vector<u8>) {
                assert!(*std::vector::borrow(&xs, 0) == 1, 0);
                assert!(*std::vector::borrow(&xs, 1) == 255, 1);
            }
        }
        }
    "#;

    let output = run_source(source, None, false);
    assert!(output.contains("Total tests: 1; passed: 1; failed: 0"));
}

#[test]
fn option_and_string_parameter_test_case_actually_executes() {
    let source = r#"
        address 0x1 {
        module M {
            use std::option::{Self, Option};
            use std::string;

            #[test(o = option::some(5), s = b"hi")]
            fun test_option_and_string(o: Option<u8>, s: vector<u8>) {
                assert!(option::is_some(&o), 0);
                assert!(*option::borrow(&o) == 5, 1);
                let s = string::utf8(s);
                assert!(string::bytes(&s) == &vector[104, 105], 2);
            }
        }
        }
    "#;

    let output = run_source(source, None, false);
    assert!(output.contains("Total tests: 1; passed: 1; failed: 0"));
}
