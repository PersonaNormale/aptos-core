// Unsuffixed and max-boundary values of u256 are both accepted.
address 0x1 {
module M {
    #[test(x = 5)]
    #[test(x = 115792089237316195423570985008687907853269984665640564039457584007913129639935u256)]
    fun u256_accepted(x: u256) {
        let _ = x;
    }
}
}
