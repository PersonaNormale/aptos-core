/// Parametric test.
///
/// Each #[test(...)] attribute is an independent test invocation of the same function.
/// The compiler emits one test case per test attribute.
/// Each test attribute executes with its own arguments.
module parametric_testing::example {
    use std::signer;

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
}
