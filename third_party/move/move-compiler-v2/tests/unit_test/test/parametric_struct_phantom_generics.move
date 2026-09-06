address 0x1 {
module M {
    struct Phantom<phantom T> has copy, drop { val: u8 }

    #[test(p = Phantom<u8> { val: 5 })]
    fun explicit_phantom_struct_param(p: Phantom<u8>) {
        let _ = p;
    }

    #[test(p = Phantom { val: 5 })]
    fun inferred_phantom_struct_param(p: Phantom<u8>) {
        let _ = p;
    }
}
}
