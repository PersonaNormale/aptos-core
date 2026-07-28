address 0x1 {
module M {
    struct Empty has drop {}

    #[test(e = Empty {})]
    fun test_empty(e: Empty) {
        let _ = e;
    }
}
}
