// A literal explicitly suffixed with one width cannot bind to a parameter of a different
// width, even if the value would otherwise fit.
address 0x1 {
module M {
    #[test(x = 5i16)]
    fun mismatch(x: i8) {
        let _ = x;
    }
}
}
