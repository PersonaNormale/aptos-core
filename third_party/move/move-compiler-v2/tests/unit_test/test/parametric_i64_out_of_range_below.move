// A negative literal is bounds-checked against its parameter's declared width too.
address 0x1 {
module M {
    #[test(x = -9223372036854775809)]
    fun out_of_range(x: i64) {
        let _ = x;
    }
}
}
