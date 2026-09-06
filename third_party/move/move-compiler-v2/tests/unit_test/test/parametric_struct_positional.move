address 0x1 {
module M {
    struct Pair(u8, u8) has copy, drop;

    #[test(p = Pair(1, 2))]
    fun positional_struct_param(p: Pair) {
        let _ = p;
    }
}
}
