// A repeated parameter assignment in a case warns; the first assignment is retained.
address 0x1 {
module M {
    #[test(addr = @0x1, addr = @0x2)]
    fun duplicate_argument(addr: signer) {
        let _ = addr;
    }
}
}
