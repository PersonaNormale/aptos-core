// A negative literal is bounds-checked against its parameter's declared width too.
address 0x1 {
module M {
    #[test(x = -32769)]
    fun out_of_range(x: i16) {
        let _ = x;
    }
}
}
