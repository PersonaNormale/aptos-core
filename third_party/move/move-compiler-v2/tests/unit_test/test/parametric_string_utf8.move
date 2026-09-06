address 0x1 {
module M {
    use std::string::{Self, String};

    #[test(s = string::utf8(vector[104, 105]))]
    fun ascii(s: String) {
        let _ = s;
    }

    #[test(s = string::utf8(vector[99, 97, 102, 195, 169]))]
    fun multi_byte(s: String) {
        let _ = s;
    }
}
}
