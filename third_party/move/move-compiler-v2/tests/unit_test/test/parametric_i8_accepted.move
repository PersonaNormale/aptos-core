// Unsuffixed, suffixed, and both boundary values of i8 are all accepted.
address 0x1 {
module M {
    #[test(x = -5)]
    #[test(x = 5i8)]
    #[test(x = 127i8)]
    #[test(x = -128i8)]
    fun i8_accepted(x: i8) {
        let _ = x;
    }
}
}
