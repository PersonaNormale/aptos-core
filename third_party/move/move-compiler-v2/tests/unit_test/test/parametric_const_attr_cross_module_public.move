address 0x1 {
module consts {
    public const LIMIT: u32 = 42;
}
module m {
    use 0x1::consts;

    #[test(x = consts::LIMIT)]
    fun t(x: u32) {
        assert!(x == 42, 0);
    }
}
}
