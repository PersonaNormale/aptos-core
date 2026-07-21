// A literal explicitly suffixed with one width cannot bind to a parameter of a different
// width, even if the value would otherwise fit.
address 0x1 {
module M {
    #[test(x = 5u16)]
    fun mismatch(x: u8) {
        let _ = x;
    }
}
}
