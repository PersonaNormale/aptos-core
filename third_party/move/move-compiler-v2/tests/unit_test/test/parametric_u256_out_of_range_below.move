// A negative literal is out of range for an unsigned parameter - reachable for the first
// time now that attribute parsing accepts a leading `-`.
address 0x1 {
module M {
    #[test(x = -1)]
    fun out_of_range(x: u256) {
        let _ = x;
    }
}
}
