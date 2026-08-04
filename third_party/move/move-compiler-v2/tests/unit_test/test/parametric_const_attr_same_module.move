address 0x1 {
module M {
    const BIG: u64 = 1000;

    #[test(x = BIG)]
    fun t(x: u64) {
        assert!(x == 1000, 0);
    }
}
}
