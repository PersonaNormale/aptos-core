// A parameterized function cannot use an empty #[test] attribute.
address 0x1 {
module M {
    #[test]
    fun empty_parameterized_case(addr: signer) {
        let _ = addr;
    }
}
}
