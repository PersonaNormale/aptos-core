// A number literal cannot be assigned to an address parameter.
address 0x1 {
module M {
    #[test(x = 5)]
    fun number_for_address(x: address) {
        let _ = x;
    }
}
}
