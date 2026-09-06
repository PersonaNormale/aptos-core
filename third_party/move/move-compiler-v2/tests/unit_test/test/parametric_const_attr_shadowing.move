address 0x1 {
module m {
    const MAX_U8: bool = false;

    #[test(x = MAX_U8)]
    fun t(x: bool) {
        assert!(!x, 0);
    }
}
}
