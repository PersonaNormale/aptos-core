// u128, the widest supported width, is accepted at its maximum value.
address 0x1 {
module M {
    #[test(x = 340282366920938463463374607431768211455u128)]
    fun u128_max(x: u128) {
        let _ = x;
    }
}
}
