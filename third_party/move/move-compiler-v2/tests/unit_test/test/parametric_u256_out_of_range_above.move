// An unsuffixed literal is still bounds-checked against its parameter's declared width.
address 0x1 {
module M {
    #[test(x = 115792089237316195423570985008687907853269984665640564039457584007913129639936)]
    fun out_of_range(x: u256) {
        let _ = x;
    }
}
}
