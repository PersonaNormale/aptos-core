// Unsuffixed, suffixed, and both boundary values of i256 are all accepted.
address 0x1 {
module M {
    #[test(x = -5)]
    #[test(x = 5i256)]
    #[test(x = 57896044618658097711785492504343953926634992332820282019728792003956564819967i256)]
    #[test(x = -57896044618658097711785492504343953926634992332820282019728792003956564819968i256)]
    fun i256_accepted(x: i256) {
        let _ = x;
    }
}
}
