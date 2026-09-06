address 0x1 {
module M {
    use std::option::{Self, Option};

    #[test(o = option::some(5u16))]
    fun wrong_arg_type(o: Option<u8>) {
        let _ = o;
    }
}
}
