address 0x1 {
module M {
    struct Pair(u8, u8) has copy, drop;

    #[test(p = Pair { first: 1, second: 2 })]
    fun named_literal_against_positional_struct(p: Pair) {
        let _ = p;
    }
}
}
