address 0x1 {
module M {
    #[test(xs = vector<u8, u16>[1, 2, 3])]
    fun bad_arity(xs: vector<u8>) {
        let _ = xs;
    }
}
}
