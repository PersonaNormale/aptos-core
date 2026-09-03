address 0x1 {
module M {
    #[test(xs = b"hi")]
    fun byte_string(xs: vector<u8>) {
        let _ = xs;
    }

    #[test(xs = x"6869")]
    fun hex_string(xs: vector<u8>) {
        let _ = xs;
    }
}
}
