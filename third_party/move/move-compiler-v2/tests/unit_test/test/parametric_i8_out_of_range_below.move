// A negative literal is bounds-checked against its parameter's declared width too - reachable
// for the first time now that attribute parsing accepts a leading `-`.
address 0x1 {
module M {
    #[test(x = -129)]
    fun out_of_range(x: i8) {
        let _ = x;
    }
}
}
