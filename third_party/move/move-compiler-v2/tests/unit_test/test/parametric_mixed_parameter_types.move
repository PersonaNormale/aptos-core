// Signer, address, and unsigned integer parameters can all be assigned in the same test attribute.
address 0x1 {
module M {
    #[test(account = @0x1, addr = @0x2, threshold = 100)]
    fun mixed(account: signer, addr: address, threshold: u64) {
        let _ = account;
        let _ = addr;
        let _ = threshold;
    }
}
}
