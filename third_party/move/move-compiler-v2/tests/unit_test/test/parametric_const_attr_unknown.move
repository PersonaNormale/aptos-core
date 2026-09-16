address 0x1 {
module m {
    #[test(x = DOES_NOT_EXIST)]
    fun t(x: u8) {
        let _ = x;
    }
}
}
