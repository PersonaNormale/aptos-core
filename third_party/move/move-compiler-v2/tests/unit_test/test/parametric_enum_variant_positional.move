address 0x1 {
module M {
    enum Either has copy, drop { Left(u8), Right(u8) }

    #[test(e = Either::Left(1))]
    fun test_enum(e: Either) {
        let _ = e;
    }
}
}
