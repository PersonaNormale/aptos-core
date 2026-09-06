address 0x1 {
module M {
    use std::ascii::{Self, Char};

    #[test(c = ascii::char(104))]
    fun valid(c: Char) {
        let _ = c;
    }
}
}
