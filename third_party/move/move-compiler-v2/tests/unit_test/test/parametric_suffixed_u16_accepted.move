// A literal suffix that agrees with the parameter's declared width is accepted.
address 0x1 {
module M {
    #[test(x = 5u16)]
    fun suffixed_u16(x: u16) {
        let _ = x;
    }
}
}
