// bool is a supported test attribute parameter type.
address 0x1 {
module M {
    #[test(flag = true)]
    fun bool_param(flag: bool) {
        let _ = flag;
    }
}
}
