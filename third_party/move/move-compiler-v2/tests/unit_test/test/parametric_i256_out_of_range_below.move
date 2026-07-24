// A negative literal is bounds-checked against its parameter's declared width too.
address 0x1 {
module M {
    #[test(x = -57896044618658097711785492504343953926634992332820282019728792003956564819969)]
    fun out_of_range(x: i256) {
        let _ = x;
    }
}
}
