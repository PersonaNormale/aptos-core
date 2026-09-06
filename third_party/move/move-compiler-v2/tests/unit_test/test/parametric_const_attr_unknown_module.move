address 0x1 {
module m {
    #[test(x = does_not_exist::CONST)]
    fun t(x: u64) {
        let _ = x;
    }
}
}
