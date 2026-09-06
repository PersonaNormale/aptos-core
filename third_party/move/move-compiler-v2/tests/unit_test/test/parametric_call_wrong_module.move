address 0x2 {
module option {
    struct FakeOption<T> has copy, drop { e: T }

    public fun some<T>(e: T): FakeOption<T> {
        FakeOption { e }
    }
}
}
address 0x1 {
module M {
    #[test(w = 0x2::option::some(1))]
    fun test_call(w: 0x2::option::FakeOption<u8>) {
        let _ = w;
    }
}
}
