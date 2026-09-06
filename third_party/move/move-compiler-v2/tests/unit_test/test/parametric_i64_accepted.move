// Unsuffixed, suffixed, and both boundary values of i64 are all accepted.
address 0x1 {
module M {
    #[test(x = -5)]
    #[test(x = 5i64)]
    #[test(x = 9223372036854775807i64)]
    #[test(x = -9223372036854775808i64)]
    fun i64_accepted(x: i64) {
        let _ = x;
    }
}
}
