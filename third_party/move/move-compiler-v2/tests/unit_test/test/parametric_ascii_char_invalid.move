address 0x1 {
module M {
    use std::ascii::{Self, Char};

    #[test(c = ascii::char(255))]
    fun invalid(c: Char) {
        let _ = c;
    }
}
}
