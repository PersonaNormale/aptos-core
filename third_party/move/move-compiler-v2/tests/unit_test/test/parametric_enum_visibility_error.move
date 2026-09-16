address 0x1 {
module Other {
    enum Private has copy, drop { Left(u8) }
}
module M {
    #[test(p = 0x1::Other::Private::Left(1))]
    fun test_private(p: 0x1::Other::Private) {
        let _ = p;
    }
}
}
