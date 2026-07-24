// Copyright (c) Aptos Foundation
// Licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

use move_core_types::{
    account_address::AccountAddress,
    int256::{I256, U256},
    value::MoveValue,
};
use move_unit_test::UnitTestingConfig;
use std::fs;
use tempfile::tempdir;

fn build_test_plan_from_source(source: &str) -> legacy_move_compiler::unit_test::TestPlan {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("argument_values.move");
    fs::write(&source_path, source).unwrap();

    let mut config = UnitTestingConfig::default()
        .with_named_addresses(move_stdlib::move_stdlib_named_addresses());
    config.source_files = vec![source_path.to_string_lossy().into_owned()];
    config.dep_files = move_stdlib::move_stdlib_files();
    config.build_test_plan().unwrap()
}

#[test]
fn unsuffixed_literal_binds_to_declared_parameter_width() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(x = 0)]
            #[test(x = 255)]
            fun bounds(x: u8) {
                let _ = x;
            }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    let case0 = module.tests.get("bounds@case0").unwrap();
    assert_eq!(case0.arguments, vec![MoveValue::U8(0)]);
    let case1 = module.tests.get("bounds@case1").unwrap();
    assert_eq!(case1.arguments, vec![MoveValue::U8(255)]);
}

#[test]
fn suffixed_literal_produces_matching_move_value_width() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(x = 5u16)]
            fun single(x: u16) {
                let _ = x;
            }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();
    let test_case = module.tests.get("single").unwrap();

    assert_eq!(test_case.arguments, vec![MoveValue::U16(5)]);
}

#[test]
fn every_unsigned_width_up_to_u128_is_supported() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(x = 1)]
            fun u8_case(x: u8) { let _ = x; }
            #[test(x = 1)]
            fun u16_case(x: u16) { let _ = x; }
            #[test(x = 1)]
            fun u32_case(x: u32) { let _ = x; }
            #[test(x = 1)]
            fun u64_case(x: u64) { let _ = x; }
            #[test(x = 1)]
            fun u128_case(x: u128) { let _ = x; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("u8_case").unwrap().arguments, vec![
        MoveValue::U8(1)
    ]);
    assert_eq!(module.tests.get("u16_case").unwrap().arguments, vec![
        MoveValue::U16(1)
    ]);
    assert_eq!(module.tests.get("u32_case").unwrap().arguments, vec![
        MoveValue::U32(1)
    ]);
    assert_eq!(module.tests.get("u64_case").unwrap().arguments, vec![
        MoveValue::U64(1)
    ]);
    assert_eq!(module.tests.get("u128_case").unwrap().arguments, vec![
        MoveValue::U128(1)
    ]);
}

#[test]
fn mixed_signer_address_and_number_parameters() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(account = @0x1, addr = @0x2, threshold = 100)]
            fun mixed(account: signer, addr: address, threshold: u64) {
                let _ = account;
                let _ = addr;
                let _ = threshold;
            }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();
    let test_case = module.tests.get("mixed").unwrap();

    assert_eq!(test_case.arguments, vec![
        MoveValue::Signer(AccountAddress::from_hex_literal("0x1").unwrap()),
        MoveValue::Address(AccountAddress::from_hex_literal("0x2").unwrap()),
        MoveValue::U64(100),
    ]);
}

#[test]
fn i8_unsuffixed_suffixed_and_boundary_values_are_supported() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(x = -5)]
            #[test(x = 5i8)]
            #[test(x = 127i8)]
            #[test(x = -128i8)]
            fun i8_accepted(x: i8) {
                let _ = x;
            }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(
        module.tests.get("i8_accepted@case0").unwrap().arguments,
        vec![MoveValue::I8(-5)]
    );
    assert_eq!(
        module.tests.get("i8_accepted@case1").unwrap().arguments,
        vec![MoveValue::I8(5)]
    );
    assert_eq!(
        module.tests.get("i8_accepted@case2").unwrap().arguments,
        vec![MoveValue::I8(127)]
    );
    assert_eq!(
        module.tests.get("i8_accepted@case3").unwrap().arguments,
        vec![MoveValue::I8(-128)]
    );
}

#[test]
fn i16_unsuffixed_suffixed_and_boundary_values_are_supported() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(x = -5)]
            #[test(x = 5i16)]
            #[test(x = 32767i16)]
            #[test(x = -32768i16)]
            fun i16_accepted(x: i16) {
                let _ = x;
            }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(
        module.tests.get("i16_accepted@case0").unwrap().arguments,
        vec![MoveValue::I16(-5)]
    );
    assert_eq!(
        module.tests.get("i16_accepted@case1").unwrap().arguments,
        vec![MoveValue::I16(5)]
    );
    assert_eq!(
        module.tests.get("i16_accepted@case2").unwrap().arguments,
        vec![MoveValue::I16(32767)]
    );
    assert_eq!(
        module.tests.get("i16_accepted@case3").unwrap().arguments,
        vec![MoveValue::I16(-32768)]
    );
}

#[test]
fn i32_unsuffixed_suffixed_and_boundary_values_are_supported() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(x = -5)]
            #[test(x = 5i32)]
            #[test(x = 2147483647i32)]
            #[test(x = -2147483648i32)]
            fun i32_accepted(x: i32) {
                let _ = x;
            }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(
        module.tests.get("i32_accepted@case0").unwrap().arguments,
        vec![MoveValue::I32(-5)]
    );
    assert_eq!(
        module.tests.get("i32_accepted@case1").unwrap().arguments,
        vec![MoveValue::I32(5)]
    );
    assert_eq!(
        module.tests.get("i32_accepted@case2").unwrap().arguments,
        vec![MoveValue::I32(2147483647)]
    );
    assert_eq!(
        module.tests.get("i32_accepted@case3").unwrap().arguments,
        vec![MoveValue::I32(-2147483648)]
    );
}

#[test]
fn i64_unsuffixed_suffixed_and_boundary_values_are_supported() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(x = -5)]
            #[test(x = 5i64)]
            #[test(x = 9223372036854775807i64)]
            #[test(x = -9223372036854775808i64)]
            fun i64_accepted(x: i64) {
                let _ = x;
            }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(
        module.tests.get("i64_accepted@case0").unwrap().arguments,
        vec![MoveValue::I64(-5)]
    );
    assert_eq!(
        module.tests.get("i64_accepted@case1").unwrap().arguments,
        vec![MoveValue::I64(5)]
    );
    assert_eq!(
        module.tests.get("i64_accepted@case2").unwrap().arguments,
        vec![MoveValue::I64(9223372036854775807)]
    );
    assert_eq!(
        module.tests.get("i64_accepted@case3").unwrap().arguments,
        vec![MoveValue::I64(-9223372036854775808)]
    );
}

#[test]
fn i128_unsuffixed_suffixed_and_boundary_values_are_supported() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(x = -5)]
            #[test(x = 5i128)]
            #[test(x = 170141183460469231731687303715884105727i128)]
            #[test(x = -170141183460469231731687303715884105728i128)]
            fun i128_accepted(x: i128) {
                let _ = x;
            }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(
        module.tests.get("i128_accepted@case0").unwrap().arguments,
        vec![MoveValue::I128(-5)]
    );
    assert_eq!(
        module.tests.get("i128_accepted@case1").unwrap().arguments,
        vec![MoveValue::I128(5)]
    );
    assert_eq!(
        module.tests.get("i128_accepted@case2").unwrap().arguments,
        vec![MoveValue::I128(170141183460469231731687303715884105727)]
    );
    assert_eq!(
        module.tests.get("i128_accepted@case3").unwrap().arguments,
        vec![MoveValue::I128(-170141183460469231731687303715884105728)]
    );
}
