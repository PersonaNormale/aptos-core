address 0x1 {
module M {
    struct Wrapper<T> has copy, drop { val: T }

    #[test(w = Wrapper<u8> { val: 5 })]
    fun explicit_generic_struct_param(w: Wrapper<u8>) {
        let _ = w;
    }

    #[test(w = Wrapper { val: 5 })]
    fun inferred_generic_struct_param(w: Wrapper<u8>) {
        let _ = w;
    }
}
}
