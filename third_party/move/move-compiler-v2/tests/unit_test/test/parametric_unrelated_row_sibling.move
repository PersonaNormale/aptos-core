// A test attribute group contains no unrelated sibling attributes.
address 0x1 {
module M {
    #[test(addr = @0x1), deprecated]
    fun unrelated_row_sibling(addr: signer) {
        let _ = addr;
    }
}
}
