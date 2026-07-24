// Unsuffixed, suffixed, and both boundary values of i32 are all accepted.
address 0x1 {
module M {
    #[test(x = -5)]
    #[test(x = 5i32)]
    #[test(x = 2147483647i32)]
    #[test(x = -2147483648i32)]
    fun i32_accepted(x: i32) {
        let _ = x;
    }
}
}
