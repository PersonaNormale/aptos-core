address 0x1 {
module M {
    enum Either has copy, drop { Left(u8), Right(u8) }

    #[test(e = Either::Left { value: 1 })]
    fun named_literal_against_positional_variant(e: Either) {
        let _ = e;
    }
}
}
