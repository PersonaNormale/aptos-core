// A no-argument test attribute also rejects unrelated sibling attributes.
address 0x1 {
module M {
    #[test, deprecated]
    fun unrelated_case_sibling_no_arg() {}
}
}
