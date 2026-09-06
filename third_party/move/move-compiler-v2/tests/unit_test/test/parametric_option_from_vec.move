// option::from_vec only exists in the framework's enum-declared Option, which this testsuite's
// default stdlib dependency (the legacy, struct-declared copy) does not provide. Attribute
// conversion only inspects the resolved function's signature, never its body, so a trivial
// inline redeclaration is enough to exercise it here.
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

    #[test(o = option::from_vec(vector[]))]
    fun from_empty_vec(o: Option<u8>) {
        let _ = o;
    }

    #[test(o = option::from_vec(vector[5]))]
    fun from_single_element_vec(o: Option<u8>) {
        let _ = o;
    }
}
}
