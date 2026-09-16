address 0x1 {
module M {
    struct Point has copy, drop { x: u8, y: u8 }

    #[test(p = Point { x: 1 })]
    fun missing_field(p: Point) {
        let _ = p;
    }
}
}
