// A reference parameter is only supported for &signer; any other reference is rejected.
address 0x1 {
module M {
    #[test(x = 5)]
    fun reference_param(x: &u8) {
        let _ = x;
    }
}
}
