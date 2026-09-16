// A multi case test warns on a parameterized top-level #[expected_failure] and drops it.
address 0x1 {
module M {
    #[test(addr = @0x1)]
    #[test(addr = @0x2)]
    #[expected_failure(abort_code = 5, location = 0x1::M)]
    fun parameterized_orphan_expected_failure(addr: signer) {
        let _ = addr;
    }
}
}
