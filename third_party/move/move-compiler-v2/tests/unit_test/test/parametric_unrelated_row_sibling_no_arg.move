// A no-argument test attribute group also rejects unrelated sibling attributes.
address 0x1 {
module M {
    #[test, deprecated]
    fun unrelated_row_sibling_no_arg() {}
}
}
