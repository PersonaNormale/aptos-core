address 0x1 {
module M {
    enum Either has copy, drop { Left(u8), Right(u8) }

    #[test(e = Either::Middle(1))]
    fun test_unknown_variant(e: Either) {
        let _ = e;
    }
}
}
