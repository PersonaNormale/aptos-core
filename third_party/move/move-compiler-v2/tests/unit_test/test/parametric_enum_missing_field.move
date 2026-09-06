address 0x1 {
module M {
    enum Shape has copy, drop { Circle { radius: u8, height: u8 } }

    #[test(s = Shape::Circle { radius: 5 })]
    fun missing_field(s: Shape) {
        let _ = s;
    }
}
}
