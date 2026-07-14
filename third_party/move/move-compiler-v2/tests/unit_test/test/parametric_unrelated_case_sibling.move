// An unrelated sibling attribute inside a test attribute warns and is dropped.
address 0x1 {
module M {
    #[test(addr = @0x1), deprecated]
    fun unrelated_case_sibling(addr: signer) {
        let _ = addr;
    }
}
}
