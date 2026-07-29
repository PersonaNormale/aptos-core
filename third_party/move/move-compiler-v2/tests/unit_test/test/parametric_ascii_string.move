address 0x1 {
module M {
    use std::ascii::{Self, String};

    #[test(s = ascii::string(vector[104, 105]))]
    fun valid(s: String) {
        let _ = s;
    }
}
}
