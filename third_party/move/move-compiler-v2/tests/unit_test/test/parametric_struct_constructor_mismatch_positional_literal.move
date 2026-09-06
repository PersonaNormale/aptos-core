address 0x1 {
module M {
    struct Point has copy, drop { x: u8, y: u8 }

    #[test(p = Point(1, 2))]
    fun positional_literal_against_named_struct(p: Point) {
        let _ = p;
    }
}
}
