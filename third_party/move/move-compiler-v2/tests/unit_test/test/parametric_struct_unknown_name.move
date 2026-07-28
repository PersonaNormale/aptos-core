address 0x1 {
module M {
    struct Point has copy, drop { x: u8 }

    #[test(p = NotAType { x: 1 })]
    fun test_typo(p: Point) {
        let _ = p;
    }
}
}
