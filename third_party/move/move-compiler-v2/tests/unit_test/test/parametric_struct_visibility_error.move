address 0x1 {
module Other {
    struct Private has copy, drop { x: u8 }
}
module M {
    #[test(p = 0x1::Other::Private { x: 1 })]
    fun test_private(p: 0x1::Other::Private) {
        let _ = p;
    }
}
}
