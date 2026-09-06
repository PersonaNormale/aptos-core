// A literal explicitly suffixed with one width cannot bind to a parameter of a different
// width, even if the value would otherwise fit.
address 0x1 {
module M {
    #[test(x = 5i8)]
    fun mismatch(x: i32) {
        let _ = x;
    }
}
}
