address 0x1 {
module m {
    const A: u8 = 1;

    #[test(xs = vector[A, A])]
    fun t(xs: vector<u8>) {
        assert!(xs == vector[1u8, 1u8], 0);
    }
}
}
