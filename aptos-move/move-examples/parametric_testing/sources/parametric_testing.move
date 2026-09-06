/// Parametric test.
///
/// Each #[test(...)] attribute is an independent test invocation of the same function.
/// The compiler emits one test case per test attribute.
/// Each test attribute executes with its own arguments.
module parametric_testing::example {
    use std::signer;
    #[test_only]
    use std::vector;

    // ---------------------------------------------------------------------------
    // Module logic under test
    // ---------------------------------------------------------------------------

    const EBLACKLISTED: u64 = 1;
    const BLACKLISTED: address = @0xBAD;

    public fun blacklisted(addr: address): bool {
        addr == BLACKLISTED
    }

    public fun owner_of(account: &signer): address {
        signer::address_of(account)
    }

    // ---------------------------------------------------------------------------
    // Signer as parameter.
    // Each test attribute runs the body independently with a different signer.
    // ---------------------------------------------------------------------------

    #[test(account = @0x1)]
    #[test(account = @0x2)]
    #[test(account = @0x3)]
    fun different_signers_are_not_blacklisted(account: signer) {
        assert!(!blacklisted(owner_of(&account)));
    }

    // ---------------------------------------------------------------------------
    // Address as parameter
    // Each test attribute runs the body independently with a different address.
    // ---------------------------------------------------------------------------

    #[test(addr = @0x1)]
    #[test(addr = @0x2)]
    #[test(addr = @0x3)]
    fun different_addresses_are_not_blacklisted(addr: address) {
        assert!(!blacklisted(addr));
    }

    // ---------------------------------------------------------------------------
    // expected_failure attribute in test attribute.
    // @case0 expects success.
    // @case1 expects the abort.
    // ---------------------------------------------------------------------------

    #[test(addr = @0x1)]
    #[test(addr = @0xBAD), expected_failure(abort_code = EBLACKLISTED)]
    fun blacklist_rejects_bad_address(addr: address) {
        assert!(!blacklisted(addr), EBLACKLISTED);
    }

    // ---------------------------------------------------------------------------
    // Every test attribute expects failure
    // ---------------------------------------------------------------------------

    #[test(addr = @0xBAD), expected_failure(abort_code = EBLACKLISTED)]
    #[test(addr = @0xBAD), expected_failure(abort_code = EBLACKLISTED)]
    fun blacklisted_always_aborts(addr: address) {
        assert!(!blacklisted(addr), EBLACKLISTED);
    }

    // ---------------------------------------------------------------------------
    // Two parameters.
    // Assignment order in the test attribute does NOT matter.
    // Execution always uses function-signature order (in this case: a first, then b).
    // ---------------------------------------------------------------------------

    #[test(b = @0x2, a = @0x1)]
    #[test(a = @0x3, b = @0x4)]
    fun order_insensitive_assignments(a: signer, b: address) {
        assert!(owner_of(&a) != b);
    }

    // ---------------------------------------------------------------------------
    // Single test attribute (no @case suffix).
    // ---------------------------------------------------------------------------

    #[test(account = @0x1)]
    fun single_case_signer(account: signer) {
        assert!(!blacklisted(owner_of(&account)));
    }

    // ---------------------------------------------------------------------------
    // Single test attribute, local expected_failure inline syntax.
    // ---------------------------------------------------------------------------

    #[test(addr = @0xBAD), expected_failure(abort_code = EBLACKLISTED)]
    fun single_case_inline_failure(addr: address) {
        assert!(!blacklisted(addr), EBLACKLISTED);
    }

    // ---------------------------------------------------------------------------
    // Legacy top-level expected_failure separate attribute syntax.
    // ---------------------------------------------------------------------------

    #[test(addr = @0xBAD)]
    #[expected_failure(abort_code = EBLACKLISTED)]
    fun single_case_legacy_failure(addr: address) {
        assert!(!blacklisted(addr), EBLACKLISTED);
    }

    // ---------------------------------------------------------------------------
    // Zero argument function single test attribute with no parameters.
    // ---------------------------------------------------------------------------

    #[test]
    fun zero_arg_case() {
        assert!(!blacklisted(@0x1));
    }

    // ---------------------------------------------------------------------------
    // Unsigned integer parameters.
    // An unsuffixed literal (e.g. `255`) is untyped and binds freely to
    // whichever unsigned width the parameter declares.
    // ---------------------------------------------------------------------------

    #[test(x = 0)]
    #[test(x = 255)]
    fun unsuffixed_literal_binds_to_declared_width(x: u8) {
        assert!(x == 0 || x == 255);
    }

    // ---------------------------------------------------------------------------
    // An explicitly suffixed literal (e.g. `42u16`) must agree with the
    // parameter's declared width; the compiler checks this before running.
    // ---------------------------------------------------------------------------

    #[test(x = 42u16)]
    fun suffixed_literal_matching_declared_width(x: u16) {
        assert!(x == 42);
    }

    // ---------------------------------------------------------------------------
    // Every unsigned width from u8 up to u128 is accepted.
    // ---------------------------------------------------------------------------

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

    // ---------------------------------------------------------------------------
    // Signer, address, and unsigned integer parameters mixed in one function.
    // ---------------------------------------------------------------------------

    #[test(account = @0x1, threshold = 100)]
    #[test(account = @0x2, threshold = 200)]
    fun mixed_signer_and_integer_params(account: signer, threshold: u64) {
        assert!(!blacklisted(owner_of(&account)));
        assert!(threshold > 0);
    }

    // ---------------------------------------------------------------------------
    // Signed integer parameters.
    // A leading `-` is recognized in attribute position, so an unsuffixed
    // negative literal binds freely to whichever signed width the parameter
    // declares, same as an unsuffixed positive literal already did.
    // ---------------------------------------------------------------------------

    #[test(x = -5)]
    #[test(x = 5)]
    fun unsuffixed_signed_literal_binds_to_declared_width(x: i8) {
        assert!(x == -5 || x == 5);
    }

    // ---------------------------------------------------------------------------
    // Every signed width from i8 up to i128 is accepted, at its own boundary.
    // ---------------------------------------------------------------------------

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

    // ---------------------------------------------------------------------------
    // 256-bit parameters, at their own boundary.
    // ---------------------------------------------------------------------------

    #[test(x = 57896044618658097711785492504343953926634992332820282019728792003956564819967i256)]
    fun i256_param_at_max_value(x: i256) {
        assert!(x == 57896044618658097711785492504343953926634992332820282019728792003956564819967);
    }

    #[test(x = 115792089237316195423570985008687907853269984665640564039457584007913129639935u256)]
    fun u256_param_at_max_value(x: u256) {
        assert!(x == 115792089237316195423570985008687907853269984665640564039457584007913129639935);
    }

    // ---------------------------------------------------------------------------
    // Signer, address, unsigned, signed, and 256-bit parameters mixed in one
    // function.
    // ---------------------------------------------------------------------------

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

    // ---------------------------------------------------------------------------
    // Bool parameter.
    // ---------------------------------------------------------------------------

    #[test(flag = true)]
    #[test(flag = false)]
    fun bool_param_accepts_true_and_false(flag: bool) {
        assert!(flag || !flag);
    }

    // ---------------------------------------------------------------------------
    // Vector-of-primitive parameter. `vector[...]` reuses Move's ordinary vector
    // literal syntax in attribute position, converting each element to the
    // element type the parameter declares.
    // ---------------------------------------------------------------------------

    #[test(xs = vector[1, 2, 3])]
    fun vector_u8_param(xs: vector<u8>) {
        assert!(vector::length(&xs) == 3);
        assert!(*vector::borrow(&xs, 0) == 1);
        assert!(*vector::borrow(&xs, 1) == 2);
        assert!(*vector::borrow(&xs, 2) == 3);
    }

    // ---------------------------------------------------------------------------
    // Vector element type may be any primitive scalar, including a signed or
    // 256-bit width, bool, or address; the same conversion each scalar
    // parameter already goes through above applies per element.
    // ---------------------------------------------------------------------------

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

    // ---------------------------------------------------------------------------
    // Nested vectors. Element parsing recurses, so a vector literal may itself
    // contain vector literals, to any depth the parameter's type declares.
    // ---------------------------------------------------------------------------

    #[test(xs = vector[vector[1, 2], vector[3]])]
    fun nested_vector_param(xs: vector<vector<u8>>) {
        assert!(vector::length(&xs) == 2);
        assert!(vector::length(vector::borrow(&xs, 0)) == 2);
        assert!(vector::length(vector::borrow(&xs, 1)) == 1);
    }

    // ---------------------------------------------------------------------------
    // Explicit element type annotation. `vector<u8>[...]` is accepted even
    // though the annotation is always redundant with the parameter's own
    // declared type; the compiler checks the two agree before running.
    // ---------------------------------------------------------------------------

    #[test(xs = vector<u8>[1, 2, 3])]
    fun explicit_vector_type_annotation(xs: vector<u8>) {
        assert!(vector::length(&xs) == 3);
    }

    // ---------------------------------------------------------------------------
    // Empty vector. The element type comes from the parameter, not the
    // (necessarily untyped) literal.
    // ---------------------------------------------------------------------------

    #[test(xs = vector[])]
    fun empty_vector_param(xs: vector<u8>) {
        assert!(vector::length(&xs) == 0);
    }

    // ---------------------------------------------------------------------------
    // Signer, address, integer, and vector parameters mixed in one function.
    // ---------------------------------------------------------------------------

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

    // ---------------------------------------------------------------------------
    // Struct parameter, named and positional forms. Both reuse Move's ordinary
    // Pack-expression grammar in attribute position.
    // ---------------------------------------------------------------------------

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

    // ---------------------------------------------------------------------------
    // Generic struct, explicit and inferred type arguments.
    // ---------------------------------------------------------------------------

    struct Wrapper<T> has copy, drop { val: T }

    #[test(w = Wrapper<u8> { val: 5 })]
    fun explicit_generic_struct_param(w: Wrapper<u8>) {
        assert!(w.val == 5);
    }

    #[test(w = Wrapper { val: 5 })]
    fun inferred_generic_struct_param(w: Wrapper<u8>) {
        assert!(w.val == 5);
    }

    // ---------------------------------------------------------------------------
    // Nesting: a struct field may be a vector, and a vector element may be a
    // struct, to any depth.
    // ---------------------------------------------------------------------------

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
}
