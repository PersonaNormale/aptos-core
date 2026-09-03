// A single case test accepts a top-level legacy #[expected_failure] in a separate attribute.
address 0x1 {
module M {
    #[test(addr = @0x1)]
    #[expected_failure]
    fun single_case_legacy_expected_failure(addr: signer) {
        let _ = addr;
        abort 1
    }
}
}
