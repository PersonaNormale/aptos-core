address 0x1 {
module M {
    #[test(xs = vector[300])]
    fun out_of_range(xs: vector<u8>) {
        let _ = xs;
    }

    #[test(xs = vector[-1])]
    fun negative_for_unsigned(xs: vector<u8>) {
        let _ = xs;
    }

    #[test(xs = vector[5u16])]
    fun suffix_mismatch(xs: vector<u8>) {
        let _ = xs;
    }
}
}
