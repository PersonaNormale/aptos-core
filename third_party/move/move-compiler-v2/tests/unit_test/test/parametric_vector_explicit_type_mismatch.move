address 0x1 {
module M {
    #[test(xs = vector<u8>[1, 2, 3])]
    fun mismatch(xs: vector<u16>) {
        let _ = xs;
    }
}
}
