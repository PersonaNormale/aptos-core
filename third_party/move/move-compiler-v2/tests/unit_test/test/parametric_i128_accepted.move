// Unsuffixed, suffixed, and both boundary values of i128 are all accepted.
address 0x1 {
module M {
    #[test(x = -5)]
    #[test(x = 5i128)]
    #[test(x = 170141183460469231731687303715884105727i128)]
    #[test(x = -170141183460469231731687303715884105728i128)]
    fun i128_accepted(x: i128) {
        let _ = x;
    }
}
}
