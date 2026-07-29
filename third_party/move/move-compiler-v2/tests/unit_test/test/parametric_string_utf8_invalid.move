address 0x1 {
module M {
    use std::string::{Self, String};

    #[test(s = string::utf8(vector[255]))]
    fun invalid_byte(s: String) {
        let _ = s;
    }
}
}
