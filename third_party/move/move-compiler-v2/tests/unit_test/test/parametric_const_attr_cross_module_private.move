address 0x1 {
module consts {
    const EINVALID_STATE: u64 = 1;
}
module m {
    #[test(x = 0x1::consts::EINVALID_STATE)]
    fun t(x: u64) {
        assert!(x == 1, 0);
    }
}
}
