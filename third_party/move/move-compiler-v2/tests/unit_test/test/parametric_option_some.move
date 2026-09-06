address 0x1 {
module M {
    use std::option::{Self, Option};

    struct Point has copy, drop { x: u8 }

    #[test(o = option::some(5))]
    fun some_primitive(o: Option<u8>) {
        let _ = o;
    }

    #[test(o = option::some(vector[1, 2, 3]))]
    fun some_vector(o: Option<vector<u8>>) {
        let _ = o;
    }

    #[test(o = option::some(Point { x: 1 }))]
    fun some_struct(o: Option<Point>) {
        let _ = o;
    }
}
}
