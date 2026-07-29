address 0x1 {
module M {
    use std::ascii::{Self, String};

    #[test(s = ascii::string(vector[255]))]
    fun invalid(s: String) {
        let _ = s;
    }
}
}
