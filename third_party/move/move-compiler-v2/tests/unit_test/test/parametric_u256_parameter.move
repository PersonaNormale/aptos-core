address 0x1 {
module M {
    #[test(x = 5u256)]
    fun u256_param(x: u256) {
        let _ = x;
    }
}
}
