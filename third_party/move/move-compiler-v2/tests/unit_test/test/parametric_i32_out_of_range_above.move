// An unsuffixed literal is still bounds-checked against its parameter's declared width.
address 0x1 {
module M {
    #[test(x = 2147483648)]
    fun out_of_range(x: i32) {
        let _ = x;
    }
}
}
