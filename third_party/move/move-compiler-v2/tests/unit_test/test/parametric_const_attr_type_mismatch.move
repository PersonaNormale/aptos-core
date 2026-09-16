address 0x1 {
module M {
    const FLAG: bool = true;

    #[test(x = FLAG)]
    fun t(x: u64) {
        let _ = x;
    }
}
}
