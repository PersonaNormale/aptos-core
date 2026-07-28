address 0x1 {
module M {
    struct Point has copy, drop { x: u8, y: u8 }
    struct Pair(u8, u8) has copy, drop;

    #[test(p = Point { x: 1 })]
    fun missing_field(p: Point) {
        let _ = p;
    }

    #[test(p = Point { x: 1, y: 2, z: 3 })]
    fun unknown_field(p: Point) {
        let _ = p;
    }

    #[test(p = Point(1, 2))]
    fun positional_literal_against_named_struct(p: Point) {
        let _ = p;
    }

    #[test(p = Pair { 0: 1, 1: 2 })]
    fun named_literal_against_positional_struct(p: Pair) {
        let _ = p;
    }
}
}
