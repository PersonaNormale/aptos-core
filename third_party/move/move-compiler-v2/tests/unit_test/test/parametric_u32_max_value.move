// u32 is accepted at its maximum value.
address 0x1 {
module M {
    #[test(x = 4294967295u32)]
    fun u32_max(x: u32) {
        let _ = x;
    }
}
}
