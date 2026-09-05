#[test_only]
/// Constants referenced from `example` to demonstrate cross-module named-constant
/// resolution in `#[test(...)]` attributes.
module parametric_testing::limits {
    public const LIMIT: u32 = 42;
    const EINVALID_STATE: u64 = 1;
}

/// Each #[test(...)] attribute invokes the function independently with its own arguments.
module parametric_testing::example {
    use std::signer;
    #[test_only]
    use std::vector;
    #[test_only]
    use std::option::{Self, Option};
    #[test_only]
    use parametric_testing::limits;

    const EBLACKLISTED: u64 = 1;
    const BLACKLISTED: address = @0xBAD;

    public fun blacklisted(addr: address): bool {
        addr == BLACKLISTED
    }

    public fun owner_of(account: &signer): address {
        signer::address_of(account)
    }

    #[test(account = @0x1)]
    #[test(account = @0x2)]
    #[test(account = @0x3)]
    fun different_signers_are_not_blacklisted(account: signer) {
        assert!(!blacklisted(owner_of(&account)));
    }

    #[test(addr = @0x1)]
    #[test(addr = @0x2)]
    #[test(addr = @0x3)]
    fun different_addresses_are_not_blacklisted(addr: address) {
        assert!(!blacklisted(addr));
    }

    // The first case succeeds; the second expects EBLACKLISTED.
    #[test(addr = @0x1)]
    #[test(addr = @0xBAD), expected_failure(abort_code = EBLACKLISTED)]
    fun blacklist_rejects_bad_address(addr: address) {
        assert!(!blacklisted(addr), EBLACKLISTED);
    }

    #[test(addr = @0xBAD), expected_failure(abort_code = EBLACKLISTED)]
    #[test(addr = @0xBAD), expected_failure(abort_code = EBLACKLISTED)]
    fun blacklisted_always_aborts(addr: address) {
        assert!(!blacklisted(addr), EBLACKLISTED);
    }

    // Assignments can appear in any order; arguments follow the function signature.
    #[test(b = @0x2, a = @0x1)]
    #[test(a = @0x3, b = @0x4)]
    fun order_insensitive_assignments(a: signer, b: address) {
        assert!(owner_of(&a) != b);
    }

    // A single test case keeps the function name without an @case suffix.
    #[test(account = @0x1)]
    fun single_case_signer(account: signer) {
        assert!(!blacklisted(owner_of(&account)));
    }

    #[test(addr = @0xBAD), expected_failure(abort_code = EBLACKLISTED)]
    fun single_case_inline_failure(addr: address) {
        assert!(!blacklisted(addr), EBLACKLISTED);
    }

    // A separate expected_failure attribute applies to a single test case.
    #[test(addr = @0xBAD)]
    #[expected_failure(abort_code = EBLACKLISTED)]
    fun single_case_legacy_failure(addr: address) {
        assert!(!blacklisted(addr), EBLACKLISTED);
    }

    #[test]
    fun zero_arg_case() {
        assert!(!blacklisted(@0x1));
    }

    // Unsuffixed literals use the declared parameter width.
    #[test(x = 0)]
    #[test(x = 255)]
    fun unsuffixed_literal_binds_to_declared_width(x: u8) {
        assert!(x == 0 || x == 255);
    }

    // An explicit numeric suffix must match the declared parameter type.
    #[test(x = 42u16)]
    fun suffixed_literal_matching_declared_width(x: u16) {
        assert!(x == 42);
    }

    #[test(x = 1)]
    fun u8_param(x: u8) {
        assert!(x == 1);
    }

    #[test(x = 1)]
    fun u32_param(x: u32) {
        assert!(x == 1);
    }

    #[test(x = 1)]
    fun u64_param(x: u64) {
        assert!(x == 1);
    }

    #[test(x = 340282366920938463463374607431768211455u128)]
    fun u128_param_at_max_value(x: u128) {
        assert!(x == 340282366920938463463374607431768211455);
    }

    #[test(account = @0x1, threshold = 100)]
    #[test(account = @0x2, threshold = 200)]
    fun mixed_signer_and_integer_params(account: signer, threshold: u64) {
        assert!(!blacklisted(owner_of(&account)));
        assert!(threshold > 0);
    }

    // Unsuffixed negative literals use the declared signed parameter width.
    #[test(x = -5)]
    #[test(x = 5)]
    fun unsuffixed_signed_literal_binds_to_declared_width(x: i8) {
        assert!(x == -5 || x == 5);
    }

    #[test(x = -128i8)]
    fun i8_param_at_min_value(x: i8) {
        assert!(x == -128);
    }

    #[test(x = 32767i16)]
    fun i16_param_at_max_value(x: i16) {
        assert!(x == 32767);
    }

    #[test(x = -2147483648i32)]
    fun i32_param_at_min_value(x: i32) {
        assert!(x == -2147483648);
    }

    #[test(x = 9223372036854775807i64)]
    fun i64_param_at_max_value(x: i64) {
        assert!(x == 9223372036854775807);
    }

    #[test(x = -170141183460469231731687303715884105728i128)]
    fun i128_param_at_min_value(x: i128) {
        assert!(x == -170141183460469231731687303715884105728);
    }

    #[test(x = 57896044618658097711785492504343953926634992332820282019728792003956564819967i256)]
    fun i256_param_at_max_value(x: i256) {
        assert!(x == 57896044618658097711785492504343953926634992332820282019728792003956564819967);
    }

    #[test(x = 115792089237316195423570985008687907853269984665640564039457584007913129639935u256)]
    fun u256_param_at_max_value(x: u256) {
        assert!(x == 115792089237316195423570985008687907853269984665640564039457584007913129639935);
    }

    #[test(account = @0x1, threshold = 100, delta = -7, big = 5u256)]
    #[test(account = @0x2, threshold = 200, delta = 7, big = 10u256)]
    fun mixed_signer_integer_signed_and_256_bit_params(
        account: signer,
        threshold: u64,
        delta: i64,
        big: u256,
    ) {
        assert!(!blacklisted(owner_of(&account)));
        assert!(threshold > 0);
        assert!(delta == -7 || delta == 7);
        assert!(big == 5 || big == 10);
    }

    #[test(flag = true)]
    #[test(flag = false)]
    fun bool_param_accepts_true_and_false(flag: bool) {
        assert!(flag || !flag);
    }

    // Vector elements use the declared parameter's element type.
    #[test(xs = vector[1, 2, 3])]
    fun vector_u8_param(xs: vector<u8>) {
        assert!(vector::length(&xs) == 3);
        assert!(*vector::borrow(&xs, 0) == 1);
        assert!(*vector::borrow(&xs, 1) == 2);
        assert!(*vector::borrow(&xs, 2) == 3);
    }

    #[test(xs = vector[-1, -2, -3])]
    fun vector_i8_param(xs: vector<i8>) {
        assert!(*vector::borrow(&xs, 0) == -1);
    }

    #[test(xs = vector[5u256, 10u256])]
    fun vector_u256_param(xs: vector<u256>) {
        assert!(*vector::borrow(&xs, 0) == 5);
        assert!(*vector::borrow(&xs, 1) == 10);
    }

    #[test(xs = vector[true, false, true])]
    fun vector_bool_param(xs: vector<bool>) {
        assert!(*vector::borrow(&xs, 0));
        assert!(!*vector::borrow(&xs, 1));
    }

    #[test(xs = vector[@0x1, @0x2])]
    fun vector_address_param(xs: vector<address>) {
        assert!(!blacklisted(*vector::borrow(&xs, 0)));
        assert!(!blacklisted(*vector::borrow(&xs, 1)));
    }

    #[test(xs = vector[vector[1, 2], vector[3]])]
    fun nested_vector_param(xs: vector<vector<u8>>) {
        assert!(vector::length(&xs) == 2);
        assert!(vector::length(vector::borrow(&xs, 0)) == 2);
        assert!(vector::length(vector::borrow(&xs, 1)) == 1);
    }

    // An explicit vector element type must match the parameter's element type.
    #[test(xs = vector<u8>[1, 2, 3])]
    fun explicit_vector_type_annotation(xs: vector<u8>) {
        assert!(vector::length(&xs) == 3);
    }

    // An empty vector gets its element type from the parameter.
    #[test(xs = vector[])]
    fun empty_vector_param(xs: vector<u8>) {
        assert!(vector::length(&xs) == 0);
    }

    #[test(account = @0x1, threshold = 100, flagged = false, xs = vector[1, 2, 3])]
    fun mixed_signer_integer_bool_and_vector_params(
        account: signer,
        threshold: u64,
        flagged: bool,
        xs: vector<u8>,
    ) {
        assert!(!blacklisted(owner_of(&account)));
        assert!(threshold > 0);
        assert!(!flagged);
        assert!(vector::length(&xs) == 3);
    }

    // Struct arguments support both named and positional literals.
    struct Point has copy, drop { x: u8, y: u8 }
    struct Pair(u8, u8) has copy, drop;

    #[test(p = Point { x: 1, y: 2 })]
    fun named_struct_param(p: Point) {
        assert!(p.x == 1);
        assert!(p.y == 2);
    }

    #[test(p = Pair(1, 2))]
    fun positional_struct_param(p: Pair) {
        assert!(p.0 == 1);
        assert!(p.1 == 2);
    }

    // Generic arguments can be explicit or inferred from the parameter type.
    struct Wrapper<T> has copy, drop { val: T }

    #[test(w = Wrapper<u8> { val: 5 })]
    fun explicit_generic_struct_param(w: Wrapper<u8>) {
        assert!(w.val == 5);
    }

    #[test(w = Wrapper { val: 5 })]
    fun inferred_generic_struct_param(w: Wrapper<u8>) {
        assert!(w.val == 5);
    }

    // Phantom type arguments are checked even though they have no runtime value.
    struct Phantom<phantom T> has copy, drop { val: u8 }

    #[test(p = Phantom<u8> { val: 5 })]
    fun explicit_phantom_struct_param(p: Phantom<u8>) {
        assert!(p.val == 5);
    }

    #[test(p = Phantom { val: 5 })]
    fun inferred_phantom_struct_param(p: Phantom<u8>) {
        assert!(p.val == 5);
    }

    struct Nested has copy, drop { pt: Point, tags: vector<u8> }

    #[test(n = Nested { pt: Point { x: 1, y: 2 }, tags: vector[9, 8] })]
    fun nested_struct_param(n: Nested) {
        assert!(n.pt.x == 1);
        assert!(vector::length(&n.tags) == 2);
    }

    #[test(v = vector[Point { x: 1, y: 2 }, Point { x: 3, y: 4 }])]
    fun vector_of_structs_param(v: vector<Point>) {
        assert!(vector::length(&v) == 2);
    }

    // A zero-field enum variant can be written without parentheses or braces.
    enum Shape has copy, drop { Circle { radius: u8 }, Square { side: u8 } }
    enum Either has copy, drop { Left(u8), Right(u8) }
    enum Color has copy, drop { Red, Green }

    #[test(s = Shape::Circle { radius: 5 })]
    fun named_variant_param(s: Shape) {
        assert!(match (s) { Shape::Circle { radius } => radius == 5, _ => false });
    }

    #[test(e = Either::Left(1))]
    fun positional_variant_param(e: Either) {
        assert!(match (e) { Either::Left(x) => x == 1, _ => false });
    }

    #[test(c = Color::Red)]
    fun bare_unit_variant_param(c: Color) {
        assert!(match (c) { Color::Red => true, Color::Green => false });
    }

    // Explicit type arguments follow the variant name: Enum::Variant<T>(...).
    enum Boxed<T> has copy, drop { Val(T) }

    #[test(b = Boxed::Val<u8>(5))]
    fun explicit_generic_variant_param(b: Boxed<u8>) {
        assert!(match (b) { Boxed::Val(v) => v == 5 });
    }

    #[test(b = Boxed::Val(5))]
    fun inferred_generic_variant_param(b: Boxed<u8>) {
        assert!(match (b) { Boxed::Val(v) => v == 5 });
    }

    enum Bundle has copy, drop { Holds(Either, vector<u8>) }

    #[test(n = Bundle::Holds(Either::Left(1), vector[9, 8]))]
    fun nested_variant_param(n: Bundle) {
        assert!(match (n) { Bundle::Holds(_, tags) => vector::length(&tags) == 2 });
    }

    // Use option::some and option::none to construct Option arguments.
    #[test(o = option::some(5))]
    fun option_some_param(o: Option<u8>) {
        assert!(option::is_some(&o));
        assert!(*option::borrow(&o) == 5);
    }

    #[test(o = option::none<u8>())]
    fun option_none_param(o: Option<u8>) {
        assert!(option::is_none(&o));
    }

    #[test(o = option::some(Point { x: 1, y: 2 }))]
    fun option_of_struct_param(o: Option<Point>) {
        assert!(option::is_some(&o));
    }

    // Byte and hex strings can supply vector<u8> arguments.
    #[test(xs = b"hi")]
    fun byte_string_literal_param(xs: vector<u8>) {
        assert!(vector::length(&xs) == 2);
    }

    #[test(xs = x"6869")]
    fun hex_string_literal_param(xs: vector<u8>) {
        assert!(vector::length(&xs) == 2);
    }

    // Test arguments can reference named constants.
    const BIG: u64 = 1000;

    #[test(x = BIG)]
    fun same_module_named_constant_param(x: u64) {
        assert!(x == 1000);
    }

    // Builtin range constants, such as MAX_U8 and MIN_I64, need no declaration.
    #[test(x = MAX_U8)]
    fun builtin_max_u8_param(x: u8) {
        assert!(x == 255);
    }

    #[test(x = MIN_I64)]
    fun builtin_min_i64_param(x: i64) {
        assert!(x == -9223372036854775808);
    }

    // Qualified constant references also support private constants in other modules.
    #[test(x = limits::LIMIT)]
    fun cross_module_public_named_constant_param(x: u32) {
        assert!(x == 42);
    }

    #[test(x = parametric_testing::limits::EINVALID_STATE)]
    fun cross_module_private_named_constant_param(x: u64) {
        assert!(x == 1);
    }

    #[test(xs = vector[BIG, BIG])]
    fun nested_in_vector_named_constant_param(xs: vector<u64>) {
        assert!(vector::length(&xs) == 2);
        assert!(*vector::borrow(&xs, 0) == 1000);
    }
}

#[test_only]
/// Shadowing demo, kept in its own module so its `MAX_U8` doesn't collide with `example`'s.
module parametric_testing::shadowing_example {
    // Local constants take precedence over builtin constants with the same name.
    const MAX_U8: bool = false;

    #[test(x = MAX_U8)]
    fun shadowing_local_constant_wins_over_builtin_param(x: bool) {
        assert!(!x);
    }
}
