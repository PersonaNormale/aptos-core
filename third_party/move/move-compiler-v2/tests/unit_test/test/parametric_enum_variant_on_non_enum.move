address 0x1 {
module M {
    struct Point has copy, drop { x: u8 }

    #[test(p = Point::Bogus(1))]
    fun test_bogus_variant(p: Point) {
        let _ = p;
    }
}
}
