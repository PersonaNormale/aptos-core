address 0x1 {
module M {
    enum Either has copy, drop { Left(u8), Right(u8) }

    #[test(e = NotAnEnum::Left(1))]
    fun test_typo(e: Either) {
        let _ = e;
    }
}
}
