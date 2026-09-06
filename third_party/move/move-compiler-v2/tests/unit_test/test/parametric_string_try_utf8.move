address 0x1 {
module M {
    use std::option::Option;
    use std::string::{Self, String};

    #[test(s = string::try_utf8(vector[104, 105]))]
    fun valid(s: Option<String>) {
        let _ = s;
    }

    #[test(s = string::try_utf8(vector[255]))]
    fun invalid(s: Option<String>) {
        let _ = s;
    }
}
}
