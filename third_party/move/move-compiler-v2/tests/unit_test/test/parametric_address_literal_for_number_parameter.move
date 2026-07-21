// An address literal cannot be assigned to an unsigned integer parameter.
address 0x1 {
module M {
    #[test(x = @0x1)]
    fun address_for_number(x: u8) {
        let _ = x;
    }
}
}
