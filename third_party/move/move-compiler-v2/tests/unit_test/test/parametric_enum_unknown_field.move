address 0x1 {
module M {
    enum Shape has copy, drop { Circle { radius: u8 } }

    #[test(s = Shape::Circle { radius: 5, extra: 1 })]
    fun unknown_field(s: Shape) {
        let _ = s;
    }
}
}
