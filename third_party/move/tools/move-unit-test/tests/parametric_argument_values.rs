// Copyright (c) Aptos Foundation
// Licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

use move_core_types::{
    account_address::AccountAddress,
    int256::{I256, U256},
    value::{MoveStruct, MoveValue},
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

#[test]
fn i256_unsuffixed_suffixed_and_boundary_values_are_supported() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(x = -5)]
            #[test(x = 5i256)]
            #[test(x = 57896044618658097711785492504343953926634992332820282019728792003956564819967i256)]
            #[test(x = -57896044618658097711785492504343953926634992332820282019728792003956564819968i256)]
            fun i256_accepted(x: i256) {
                let _ = x;
            }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(
        module.tests.get("i256_accepted@case0").unwrap().arguments,
        vec![MoveValue::I256(I256::from(-5i64))]
    );
    assert_eq!(
        module.tests.get("i256_accepted@case1").unwrap().arguments,
        vec![MoveValue::I256(I256::from(5i64))]
    );
    assert_eq!(
        module.tests.get("i256_accepted@case2").unwrap().arguments,
        vec![MoveValue::I256(I256::MAX)]
    );
    assert_eq!(
        module.tests.get("i256_accepted@case3").unwrap().arguments,
        vec![MoveValue::I256(I256::MIN)]
    );
}

#[test]
fn u256_unsuffixed_suffixed_and_max_boundary_are_supported() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(x = 5)]
            #[test(x = 115792089237316195423570985008687907853269984665640564039457584007913129639935u256)]
            fun u256_accepted(x: u256) {
                let _ = x;
            }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(
        module.tests.get("u256_accepted@case0").unwrap().arguments,
        vec![MoveValue::U256(U256::from(5u64))]
    );
    assert_eq!(
        module.tests.get("u256_accepted@case1").unwrap().arguments,
        vec![MoveValue::U256(U256::MAX)]
    );
}

#[test]
fn mixed_signer_address_number_signed_and_256_bit_parameters() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(account = @0x1, addr = @0x2, threshold = 100, delta = -7, big = 5u256)]
            fun mixed(
                account: signer,
                addr: address,
                threshold: u64,
                delta: i64,
                big: u256,
            ) {
                let _ = account;
                let _ = addr;
                let _ = threshold;
                let _ = delta;
                let _ = big;
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
        MoveValue::I64(-7),
        MoveValue::U256(U256::from(5u64)),
    ]);
}

#[test]
fn bool_true_and_false_are_supported() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(b = true)]
            #[test(b = false)]
            fun bool_accepted(b: bool) {
                let _ = b;
            }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(
        module.tests.get("bool_accepted@case0").unwrap().arguments,
        vec![MoveValue::Bool(true)]
    );
    assert_eq!(
        module.tests.get("bool_accepted@case1").unwrap().arguments,
        vec![MoveValue::Bool(false)]
    );
}

#[test]
fn vector_of_each_scalar_primitive_is_supported() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(xs = vector[1, 2, 3])]
            fun u8_vec(xs: vector<u8>) { let _ = xs; }
            #[test(xs = vector[1, 2, 3])]
            fun u256_vec(xs: vector<u256>) { let _ = xs; }
            #[test(xs = vector[-1, -2, -3])]
            fun i8_vec(xs: vector<i8>) { let _ = xs; }
            #[test(xs = vector[-1, -2, -3])]
            fun i256_vec(xs: vector<i256>) { let _ = xs; }
            #[test(xs = vector[true, false])]
            fun bool_vec(xs: vector<bool>) { let _ = xs; }
            #[test(xs = vector[@0x1, @0x2])]
            fun address_vec(xs: vector<address>) { let _ = xs; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("u8_vec").unwrap().arguments, vec![
        MoveValue::Vector(vec![MoveValue::U8(1), MoveValue::U8(2), MoveValue::U8(3),])
    ]);
    assert_eq!(module.tests.get("u256_vec").unwrap().arguments, vec![
        MoveValue::Vector(vec![
            MoveValue::U256(U256::from(1u64)),
            MoveValue::U256(U256::from(2u64)),
            MoveValue::U256(U256::from(3u64)),
        ])
    ]);
    assert_eq!(module.tests.get("i8_vec").unwrap().arguments, vec![
        MoveValue::Vector(vec![
            MoveValue::I8(-1),
            MoveValue::I8(-2),
            MoveValue::I8(-3),
        ])
    ]);
    assert_eq!(module.tests.get("i256_vec").unwrap().arguments, vec![
        MoveValue::Vector(vec![
            MoveValue::I256(I256::from(-1i64)),
            MoveValue::I256(I256::from(-2i64)),
            MoveValue::I256(I256::from(-3i64)),
        ])
    ]);
    assert_eq!(module.tests.get("bool_vec").unwrap().arguments, vec![
        MoveValue::Vector(vec![MoveValue::Bool(true), MoveValue::Bool(false),])
    ]);
    assert_eq!(module.tests.get("address_vec").unwrap().arguments, vec![
        MoveValue::Vector(vec![
            MoveValue::Address(AccountAddress::from_hex_literal("0x1").unwrap()),
            MoveValue::Address(AccountAddress::from_hex_literal("0x2").unwrap()),
        ])
    ]);
}

#[test]
fn explicit_vector_type_annotation_matching_parameter_is_supported() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(xs = vector<u8>[1, 2, 3])]
            fun explicit(xs: vector<u8>) {
                let _ = xs;
            }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("explicit").unwrap().arguments, vec![
        MoveValue::Vector(vec![MoveValue::U8(1), MoveValue::U8(2), MoveValue::U8(3),])
    ]);
}

#[test]
fn empty_vector_is_supported_for_every_element_type() {
    let source = r#"
        address 0x1 {
        module M {
            #[test(xs = vector[])]
            fun empty_u8(xs: vector<u8>) { let _ = xs; }
            #[test(xs = vector[])]
            fun empty_bool(xs: vector<bool>) { let _ = xs; }
            #[test(xs = vector[])]
            fun empty_address(xs: vector<address>) { let _ = xs; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("empty_u8").unwrap().arguments, vec![
        MoveValue::Vector(vec![])
    ]);
    assert_eq!(module.tests.get("empty_bool").unwrap().arguments, vec![
        MoveValue::Vector(vec![])
    ]);
    assert_eq!(module.tests.get("empty_address").unwrap().arguments, vec![
        MoveValue::Vector(vec![])
    ]);
}

#[test]
fn named_struct_same_module_is_supported() {
    let source = r#"
        address 0x1 {
        module M {
            struct Point has copy, drop { x: u8, y: u8 }
            #[test(p = Point { x: 1, y: 2 })]
            fun test_point(p: Point) { let _ = p; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_point").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new(vec![MoveValue::U8(1), MoveValue::U8(2)]))
    ]);
}

#[test]
fn named_struct_field_order_independent() {
    let source = r#"
        address 0x1 {
        module M {
            struct Point has copy, drop { x: u8, y: u8 }
            #[test(p = Point { y: 2, x: 1 })]
            fun test_point(p: Point) { let _ = p; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_point").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new(vec![MoveValue::U8(1), MoveValue::U8(2)]))
    ]);
}

#[test]
fn positional_struct_same_module_is_supported() {
    let source = r#"
        address 0x1 {
        module M {
            struct Pair(u8, u8) has copy, drop;
            #[test(p = Pair(1, 2))]
            fun test_pair(p: Pair) { let _ = p; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_pair").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new(vec![MoveValue::U8(1), MoveValue::U8(2)]))
    ]);
}

#[test]
fn generic_struct_explicit_type_args_matching_parameter() {
    let source = r#"
        address 0x1 {
        module M {
            struct Wrapper<T> has copy, drop { val: T }
            #[test(w = Wrapper<u8> { val: 5 })]
            fun test_wrapper(w: Wrapper<u8>) { let _ = w; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_wrapper").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new(vec![MoveValue::U8(5)]))
    ]);
}

#[test]
fn generic_struct_type_args_inferred_from_parameter() {
    let source = r#"
        address 0x1 {
        module M {
            struct Wrapper<T> has copy, drop { val: T }
            #[test(w = Wrapper { val: 5 })]
            fun test_wrapper(w: Wrapper<u8>) { let _ = w; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_wrapper").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new(vec![MoveValue::U8(5)]))
    ]);
}

#[test]
fn generic_struct_two_type_parameters_one_used_per_field() {
    let source = r#"
        address 0x1 {
        module M {
            struct Both<A, B> has copy, drop { first: A, second: B }
            #[test(b = Both<u8, bool> { first: 5, second: true })]
            fun test_both(b: Both<u8, bool>) { let _ = b; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_both").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new(vec![
            MoveValue::U8(5),
            MoveValue::Bool(true)
        ]))
    ]);
}

#[test]
fn phantom_struct_explicit_type_args_matching_parameter() {
    let source = r#"
        address 0x1 {
        module M {
            struct Phantom<phantom T> has copy, drop { val: u8 }
            #[test(p = Phantom<u8> { val: 5 })]
            fun test_phantom(p: Phantom<u8>) { let _ = p; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_phantom").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new(vec![MoveValue::U8(5)]))
    ]);
}

#[test]
fn phantom_struct_type_args_inferred_from_parameter() {
    let source = r#"
        address 0x1 {
        module M {
            struct Phantom<phantom T> has copy, drop { val: u8 }
            #[test(p = Phantom { val: 5 })]
            fun test_phantom(p: Phantom<u8>) { let _ = p; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_phantom").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new(vec![MoveValue::U8(5)]))
    ]);
}

#[test]
fn struct_field_containing_a_vector() {
    let source = r#"
        address 0x1 {
        module M {
            struct Tagged has copy, drop { tags: vector<u8> }
            #[test(t = Tagged { tags: vector[9, 8] })]
            fun test_tagged(t: Tagged) { let _ = t; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_tagged").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new(vec![MoveValue::Vector(vec![
            MoveValue::U8(9),
            MoveValue::U8(8)
        ])]))
    ]);
}

#[test]
fn struct_field_containing_another_struct() {
    let source = r#"
        address 0x1 {
        module M {
            struct Point has copy, drop { x: u8, y: u8 }
            struct Nested has copy, drop { pt: Point }
            #[test(n = Nested { pt: Point { x: 1, y: 2 } })]
            fun test_nested(n: Nested) { let _ = n; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_nested").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new(vec![MoveValue::Struct(MoveStruct::new(
            vec![MoveValue::U8(1), MoveValue::U8(2)]
        ))]))
    ]);
}

#[test]
fn vector_element_containing_a_struct() {
    let source = r#"
        address 0x1 {
        module M {
            struct Point has copy, drop { x: u8, y: u8 }
            #[test(v = vector[Point { x: 1, y: 2 }, Point { x: 3, y: 4 }])]
            fun test_vec(v: vector<Point>) { let _ = v; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_vec").unwrap().arguments, vec![
        MoveValue::Vector(vec![
            MoveValue::Struct(MoveStruct::new(vec![MoveValue::U8(1), MoveValue::U8(2)])),
            MoveValue::Struct(MoveStruct::new(vec![MoveValue::U8(3), MoveValue::U8(4)])),
        ])
    ]);
}

#[test]
fn three_levels_of_nesting() {
    let source = r#"
        address 0x1 {
        module M {
            struct Point has copy, drop { x: u8, y: u8 }
            struct Nested has copy, drop { pt: Point, tags: vector<u8> }
            #[test(n = Nested { pt: Point { x: 1, y: 2 }, tags: vector[9, 8] })]
            fun test_nested(n: Nested) { let _ = n; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_nested").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new(vec![
            MoveValue::Struct(MoveStruct::new(vec![MoveValue::U8(1), MoveValue::U8(2)])),
            MoveValue::Vector(vec![MoveValue::U8(9), MoveValue::U8(8)]),
        ]))
    ]);
}

#[test]
fn public_struct_constructible_from_another_module() {
    let source = r#"
        address 0x1 {
        module Defines {
            public struct Point has copy, drop { x: u8 }
        }
        module M {
            use 0x1::Defines::Point;
            #[test(p = Point { x: 1 })]
            fun test_point(p: Point) { let _ = p; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan
        .module_tests
        .iter()
        .find(|(id, _)| id.short_str_lossless().contains("M"))
        .map(|(_, m)| m)
        .unwrap();

    assert_eq!(module.tests.get("test_point").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new(vec![MoveValue::U8(1)]))
    ]);
}

#[test]
fn named_variant_same_module_is_supported() {
    let source = r#"
        address 0x1 {
        module M {
            enum Shape has copy, drop { Circle { radius: u8 }, Square { side: u8 } }
            #[test(s = Shape::Circle { radius: 5 })]
            fun test_shape(s: Shape) { let _ = s; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_shape").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new_variant(0, vec![MoveValue::U8(5)]))
    ]);
}

#[test]
fn named_variant_field_order_independent() {
    let source = r#"
        address 0x1 {
        module M {
            enum Shape has copy, drop { Circle { radius: u8, extra: u8 } }
            #[test(s = Shape::Circle { extra: 9, radius: 5 })]
            fun test_shape(s: Shape) { let _ = s; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_shape").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new_variant(0, vec![
            MoveValue::U8(5),
            MoveValue::U8(9)
        ]))
    ]);
}

#[test]
fn positional_variant_same_module_is_supported() {
    let source = r#"
        address 0x1 {
        module M {
            enum Either has copy, drop { Left(u8), Right(u8) }
            #[test(e = Either::Left(1))]
            fun test_either(e: Either) { let _ = e; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_either").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new_variant(0, vec![MoveValue::U8(1)]))
    ]);
}

#[test]
fn bare_unit_variant_is_supported() {
    let source = r#"
        address 0x1 {
        module M {
            enum Color has copy, drop { Red, Green }
            #[test(c = Color::Green)]
            fun test_color(c: Color) { let _ = c; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_color").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new_variant(1, vec![]))
    ]);
}

#[test]
fn unit_variant_explicit_empty_parens_and_braces_agree() {
    let source = r#"
        address 0x1 {
        module M {
            enum Color has copy, drop { Red, Green }
            #[test(a = Color::Red)]
            fun bare(a: Color) { let _ = a; }
            #[test(a = Color::Red())]
            fun parens(a: Color) { let _ = a; }
            #[test(a = Color::Red {})]
            fun braces(a: Color) { let _ = a; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    let expected = vec![MoveValue::Struct(MoveStruct::new_variant(0, vec![]))];
    assert_eq!(
        module.tests.get("bare").unwrap().arguments,
        expected.clone()
    );
    assert_eq!(
        module.tests.get("parens").unwrap().arguments,
        expected.clone()
    );
    assert_eq!(module.tests.get("braces").unwrap().arguments, expected);
}

#[test]
fn generic_enum_variant_explicit_type_args_matching_parameter() {
    let source = r#"
        address 0x1 {
        module M {
            enum Wrapper<T> has copy, drop { Val(T) }
            #[test(w = Wrapper::Val<u8>(5))]
            fun test_wrapper(w: Wrapper<u8>) { let _ = w; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_wrapper").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new_variant(0, vec![MoveValue::U8(5)]))
    ]);
}

#[test]
fn generic_enum_variant_type_args_inferred_from_parameter() {
    let source = r#"
        address 0x1 {
        module M {
            enum Wrapper<T> has copy, drop { Val(T) }
            #[test(w = Wrapper::Val(5))]
            fun test_wrapper(w: Wrapper<u8>) { let _ = w; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_wrapper").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new_variant(0, vec![MoveValue::U8(5)]))
    ]);
}

#[test]
fn generic_enum_variant_two_type_parameters_one_used_per_field() {
    let source = r#"
        address 0x1 {
        module M {
            enum Both<A, B> has copy, drop { Pair(A, B) }
            #[test(b = Both::Pair<u8, bool>(5, true))]
            fun test_both(b: Both<u8, bool>) { let _ = b; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_both").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new_variant(0, vec![
            MoveValue::U8(5),
            MoveValue::Bool(true)
        ]))
    ]);
}

#[test]
fn variant_field_containing_a_vector() {
    let source = r#"
        address 0x1 {
        module M {
            enum Tagged has copy, drop { Tags(vector<u8>) }
            #[test(t = Tagged::Tags(vector[9, 8]))]
            fun test_tagged(t: Tagged) { let _ = t; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_tagged").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new_variant(0, vec![MoveValue::Vector(vec![
            MoveValue::U8(9),
            MoveValue::U8(8)
        ])]))
    ]);
}

#[test]
fn variant_field_containing_another_variant() {
    let source = r#"
        address 0x1 {
        module M {
            enum Either has copy, drop { Left(u8), Right(u8) }
            enum Nested has copy, drop { Holds(Either) }
            #[test(n = Nested::Holds(Either::Left(1)))]
            fun test_nested(n: Nested) { let _ = n; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_nested").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new_variant(0, vec![MoveValue::Struct(
            MoveStruct::new_variant(0, vec![MoveValue::U8(1)])
        )]))
    ]);
}

#[test]
fn vector_element_containing_a_variant() {
    let source = r#"
        address 0x1 {
        module M {
            enum Either has copy, drop { Left(u8), Right(u8) }
            #[test(v = vector[Either::Left(1), Either::Right(2)])]
            fun test_vec(v: vector<Either>) { let _ = v; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_vec").unwrap().arguments, vec![
        MoveValue::Vector(vec![
            MoveValue::Struct(MoveStruct::new_variant(0, vec![MoveValue::U8(1)])),
            MoveValue::Struct(MoveStruct::new_variant(1, vec![MoveValue::U8(2)])),
        ])
    ]);
}

#[test]
fn three_levels_of_nesting_with_a_variant() {
    let source = r#"
        address 0x1 {
        module M {
            enum Either has copy, drop { Left(u8), Right(u8) }
            enum Nested has copy, drop { Holds(Either, vector<u8>) }
            #[test(n = Nested::Holds(Either::Left(1), vector[9, 8]))]
            fun test_nested(n: Nested) { let _ = n; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_nested").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new_variant(0, vec![
            MoveValue::Struct(MoveStruct::new_variant(0, vec![MoveValue::U8(1)])),
            MoveValue::Vector(vec![MoveValue::U8(9), MoveValue::U8(8)]),
        ]))
    ]);
}

#[test]
fn struct_field_containing_a_variant() {
    let source = r#"
        address 0x1 {
        module M {
            enum Either has copy, drop { Left(u8), Right(u8) }
            struct Holder has copy, drop { inner: Either }
            #[test(h = Holder { inner: Either::Left(1) })]
            fun test_holder(h: Holder) { let _ = h; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan.module_tests.values().next().unwrap();

    assert_eq!(module.tests.get("test_holder").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new(vec![MoveValue::Struct(
            MoveStruct::new_variant(0, vec![MoveValue::U8(1)])
        )]))
    ]);
}

#[test]
fn public_enum_variant_constructible_from_another_module() {
    let source = r#"
        address 0x1 {
        module Defines {
            public enum Either has copy, drop { Left(u8), Right(u8) }
        }
        module M {
            use 0x1::Defines::Either;
            #[test(e = Either::Left(1))]
            fun test_either(e: Either) { let _ = e; }
        }
        }
    "#;

    let plan = build_test_plan_from_source(source);
    let module = plan
        .module_tests
        .iter()
        .find(|(id, _)| id.short_str_lossless().contains("M"))
        .map(|(_, m)| m)
        .unwrap();

    assert_eq!(module.tests.get("test_either").unwrap().arguments, vec![
        MoveValue::Struct(MoveStruct::new_variant(0, vec![MoveValue::U8(1)]))
    ]);
}
