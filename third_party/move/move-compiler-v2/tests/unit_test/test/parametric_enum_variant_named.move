address 0x1 {
module M {
    enum Shape has copy, drop { Circle { radius: u8 }, Square { side: u8 } }

    #[test(s = Shape::Circle { radius: 5 })]
    fun named_variant_param(s: Shape) {
        let _ = s;
    }
}
}
