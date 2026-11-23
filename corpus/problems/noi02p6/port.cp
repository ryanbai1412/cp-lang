// Source: NOI 2002 P6 (Robot No. M), https://dmoj.ca/problem/noi02p6
// Problem: m = prod p_i^e_i is given factored (k <= 1000 primes < 1e4, e_i <= 1e6). Robot m's
//   teachers are its divisors other than 1 and m; robot d's independence number counts the
//   smaller robots sharing no teacher with it. Over m and its teachers, sum that number for d
//   a product of an even count of distinct odd primes, of an odd count, and the rest, mod 1e4.
// Solution: the independence number of d is phi(d). The first two classes are squarefree odd
//   d, where phi(d) = prod (p - 1): track the even- and odd-count sums over the odd primes
//   (d = 1 counts as even, so take it off). phi summed over all divisors is m, so the rest is
//   m - even - odd. O(k log e).
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
