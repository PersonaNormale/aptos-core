address 0x1 {
module M {
    enum Shape has copy, drop { Circle { radius: u8 } }

    #[test(s = Shape::Circle(5))]
    fun positional_literal_against_named_variant(s: Shape) {
        let _ = s;
    }
}
}
