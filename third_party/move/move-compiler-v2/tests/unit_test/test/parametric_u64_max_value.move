// u64 is accepted at its maximum value.
address 0x1 {
module M {
    #[test(x = 18446744073709551615u64)]
    fun u64_max(x: u64) {
        let _ = x;
    }
}
}
