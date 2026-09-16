// Single case tests keep the function name without an @case suffix.
address 0x1 {
module M {
    #[test(addr = @0x1)]
    fun single_case_test(addr: signer) {
        let _ = addr;
    }
}
}
