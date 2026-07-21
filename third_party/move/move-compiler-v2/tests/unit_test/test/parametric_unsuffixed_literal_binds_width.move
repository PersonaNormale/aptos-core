// An unsuffixed literal is untyped and binds freely to the parameter's declared width.
address 0x1 {
module M {
    #[test(x = 0)]
    #[test(x = 255)]
    fun unsuffixed_u8(x: u8) {
        let _ = x;
    }
}
}
