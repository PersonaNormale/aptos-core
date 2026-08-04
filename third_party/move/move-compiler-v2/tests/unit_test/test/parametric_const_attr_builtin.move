address 0x1 {
module m {
    #[test(x = MAX_U8)]
    fun u8_max(x: u8) {
        assert!(x == 255, 0);
    }

    #[test(x = MIN_I64)]
    fun i64_min(x: i64) {
        let _ = x;
    }
}
}
