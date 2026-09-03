address 0x1 {
module M {
    struct Point has copy, drop { x: u8, y: u8 }

    #[test(p = Point { x: 1, y: 2 })]
    fun named_struct_param(p: Point) {
        let _ = p;
    }
}
}
