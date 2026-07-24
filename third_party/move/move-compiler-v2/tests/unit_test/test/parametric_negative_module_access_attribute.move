// A leading `-` not followed by a standalone numeral (here, a module access) still falls back
// to the parser's existing diagnostic instead of being treated as a negative literal or
// panicking.
address 0x1 {
module M {
    #[test(x = -0x1::M)]
    fun f(x: u8) {
        let _ = x;
    }
}
}
