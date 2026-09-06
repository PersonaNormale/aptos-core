address 0x1 {
module M {
    use std::option::{Self, Option};

    #[test(o = option::some { e: 1 })]
    fun named_call(o: Option<u8>) {
        let _ = o;
    }
}
}
