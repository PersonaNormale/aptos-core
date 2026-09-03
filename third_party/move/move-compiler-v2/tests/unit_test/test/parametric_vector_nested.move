address 0x1 {
module M {
    #[test(xs = vector[vector[1, 2], vector[3]])]
    fun depth2(xs: vector<vector<u8>>) {
        let _ = xs;
    }

    #[test(xs = vector[vector[vector[1]], vector[vector[2], vector[3]]])]
    fun depth3(xs: vector<vector<vector<u8>>>) {
        let _ = xs;
    }

    #[test(xs = vector[])]
    fun empty_outer(xs: vector<vector<u8>>) {
        let _ = xs;
    }

    #[test(xs = vector[vector[]])]
    fun empty_inner(xs: vector<vector<u8>>) {
        let _ = xs;
    }
}
}
