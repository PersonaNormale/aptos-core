address 0x1 {
module M {
    struct Wrapper has copy, drop { x: u8 }

    public fun helper(x: u8): u8 { x }

    #[test(w = helper(1))]
    fun test_call(w: Wrapper) {
        let _ = w;
    }
}
}
