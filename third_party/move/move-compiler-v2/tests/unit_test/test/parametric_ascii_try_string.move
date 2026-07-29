address 0x1 {
module M {
    use std::ascii;
    use std::option::Option;

    #[test(s = ascii::try_string(vector[104, 105]))]
    fun valid(s: Option<ascii::String>) {
        let _ = s;
    }

    #[test(s = ascii::try_string(vector[255]))]
    fun invalid(s: Option<ascii::String>) {
        let _ = s;
    }
}
}
