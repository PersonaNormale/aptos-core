// Unsuffixed, suffixed, and both boundary values of i16 are all accepted.
address 0x1 {
module M {
    #[test(x = -5)]
    #[test(x = 5i16)]
    #[test(x = 32767i16)]
    #[test(x = -32768i16)]
    fun i16_accepted(x: i16) {
        let _ = x;
    }
}
}
