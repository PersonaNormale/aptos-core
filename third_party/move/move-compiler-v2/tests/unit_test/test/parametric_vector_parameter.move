address 0x1 {
module M {
    #[test(xs = vector[1, 2, 3])]
    fun vector_param(xs: vector<u8>) {
        let _ = xs;
    }
}
}
