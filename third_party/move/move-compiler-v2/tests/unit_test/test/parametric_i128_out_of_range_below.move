// A literal whose magnitude resolves it to a wider integer type cannot bind to a narrower
// parameter, even if the value would otherwise fit the parameter's sign.
address 0x1 {
module M {
    #[test(x = -170141183460469231731687303715884105729)]
    fun out_of_range(x: i128) {
        let _ = x;
    }
}
}
