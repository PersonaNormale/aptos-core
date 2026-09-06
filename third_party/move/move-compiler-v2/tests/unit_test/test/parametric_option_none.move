address 0x1 {
module M {
    use std::option::{Self, Option};

    struct Point has copy, drop { x: u8 }

    #[test(o = option::none<u8>())]
    fun none_primitive(o: Option<u8>) {
        let _ = o;
    }

    #[test(o = option::none<Point>())]
    fun none_struct(o: Option<Point>) {
        let _ = o;
    }
}
}
