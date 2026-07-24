// An unsuffixed literal is still bounds-checked against its parameter's declared width.
address 0x1 {
module M {
    #[test(x = 9223372036854775808)]
    fun out_of_range(x: i64) {
        let _ = x;
    }
}
}
