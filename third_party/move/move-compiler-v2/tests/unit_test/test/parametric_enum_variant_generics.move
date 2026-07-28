address 0x1 {
module M {
    enum Wrapper<T> has copy, drop { Val(T) }

    #[test(w = Wrapper::Val<u8>(5))]
    fun explicit_type_args(w: Wrapper<u8>) {
        let _ = w;
    }

    #[test(w = Wrapper::Val(5))]
    fun inferred_type_args(w: Wrapper<u8>) {
        let _ = w;
    }
}
}
