// An unsuffixed literal is still bounds-checked against its parameter's declared width.
address 0x1 {
module M {
    #[test(x = 170141183460469231731687303715884105728)]
    fun out_of_range(x: i128) {
        let _ = x;
    }
}
}
