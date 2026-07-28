address 0x1 {
module M {
    struct Point has copy, drop { x: u8, y: u8 }

    #[test(p = Point { x: 1, y: 2, z: 3 })]
    fun unknown_field(p: Point) {
        let _ = p;
    }
}
}
