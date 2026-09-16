address 0x1 {
module M {
    #[test(addr = @0x1)]
    #[test(addr = @0x2)]
    fun test_two_cases(addr: signer) {
        let _ = addr;
    }
}
}
