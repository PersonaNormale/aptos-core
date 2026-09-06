// no-stdlib
address 0x1 {
module option {
    enum Option<Element> has copy, drop, store {
        None,
        Some { e: Element },
    }
    public fun from_vec<Element>(vec: vector<Element>): Option<Element> {
        abort 0
    }
}
module M {
    use 0x1::option::{Self, Option};

    #[test(o = option::from_vec(vector[1, 2, 3]))]
    fun too_long(o: Option<u8>) {
        let _ = o;
    }
}
}
