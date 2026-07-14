// A test attribute contains no unrelated sibling attributes.
address 0x1 {
module M {
    #[test(addr = @0x1), deprecated]
    fun unrelated_case_sibling(addr: signer) {
        let _ = addr;
    }
}
}
