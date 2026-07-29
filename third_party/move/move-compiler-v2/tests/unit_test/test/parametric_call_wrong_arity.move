address 0x1 {
module M {
    use std::option::{Self, Option};

    #[test(o = option::some())]
    fun wrong_arity(o: Option<u8>) {
        let _ = o;
    }
}
}
