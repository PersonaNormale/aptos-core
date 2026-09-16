address 0x1 {
module M {
    enum Color has copy, drop { Red, Green }

    #[test(c = Color::Red)]
    fun bare_unit_variant(c: Color) {
        let _ = c;
    }

    #[test(c = Color::Red())]
    fun explicit_positional_empty(c: Color) {
        let _ = c;
    }

    #[test(c = Color::Green {})]
    fun explicit_named_empty(c: Color) {
        let _ = c;
    }
}
}
