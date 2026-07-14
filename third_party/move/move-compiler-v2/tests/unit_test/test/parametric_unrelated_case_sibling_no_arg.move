// A no-argument test attribute also warns on unrelated sibling attributes, and drops them.
address 0x1 {
module M {
    #[test, deprecated]
    fun unrelated_case_sibling_no_arg() {}
}
}
