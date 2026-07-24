// An unsuffixed literal is still bounds-checked against its parameter's declared width.
address 0x1 {
module M {
    #[test(x = 128)]
    fun out_of_range(x: i8) {
        let _ = x;
    }
}
}
