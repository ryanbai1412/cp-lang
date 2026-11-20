// Runnable today/ port of NOI '02 P6 (Robot M) for real Rust `cpl build`.
// Stdin: n, then n pairs (p_i, e_i). Prints independent, odd-prime-parity, dependent counts mod 10000.
// original: corpus/submissions/3754486.cpp

fn main_solve():
    let modv = 10_000
    let n: Int = read()
    var even = 1
    var odd = 0
    var total = 1

    fn mpow(base: Int, exp: Int) -> Int:
        var res = 1
        var b = base % modv
        var e = exp
        while e > 0:
            if (e & 1) == 1: res = (res * b) % modv
            b = (b * b) % modv
            e = e >> 1
        return res

    for _ in 0..n:
        let p: Int = read()
        let e: Int = read()
        total = (total * mpow(p, e)) % modv
        if p > 2:
            let ne = (even + odd * (p - 1)) % modv
            let no = (odd + even * (p - 1)) % modv
            even = ne
            odd = no

    print((even - 1 + modv) % modv)
    print(odd)
    print((total - even - odd + 2 * modv) % modv)

main_solve()
