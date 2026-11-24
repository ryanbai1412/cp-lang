// Source: Codeforces 2262E (Paired Bracket Sequences),
//   https://codeforces.com/contest/2262/problem/E
// limits: 2000 ms, 256 MB
// Problem: for each k = 0..n, count ordered pairs (s, t) of balanced bracket sequences of length
//   2n with exactly k common pairings (matched pairs (i, j) in both), modulo a prime M in
//   [1e8, 1e9]. n <= 500, multitest, sum n <= 500.
// Solution: ans[k] counts pairs with k marked common pairings: the marks cut the sequence into
//   k + 1 balanced regions, a region of length 2i has C_i^2 choices of (s, t), and Lagrange
//   inversion gives ans[k] = C(2n, k) / (k + 1) [u^(n+1)] S^(k+1), S = sum C_i^2 u^(i+1).
//   Powers of S by convolve modulo three NTT primes and CRT, O(n^2 log n); then exact
//   counts by inclusion-exclusion, out[j] = sum_k (-1)^(k-j) C(k, j) ans[k], O(n^2).

use std.ntt.{ntt}

@!multitest
let (n, m): (1..=500, 2..=2 * 10 ** 9) = read()
type Z = Mod<m>
let comb = Comb<Z>()

// convolve over Mod<m> has no NTT for an arbitrary prime m (it runs its O(n^2) loop), and the
// exact Int products reach 5e20, past Int; so convolve the residues modulo three NTT primes and
// rebuild each coefficient modulo m by Garner (the true value is under p1 p2 p3 ~ 7.9e25).
// The powers only need their first n + 2 terms, so one fixed transform length works throughout
// and the transform of `symbol` is taken once per prime.
let size = bit_ceil(2 * (n + 2))
fn transform<P>(_: Mod<P>, a: List<Int>) -> List<Mod<P>>:
    var t = List.filled(size, Mod<P>(0))
    for (i, x) in a.items: t[i] = Mod<P>(x)
    ntt(mut t, bit_reversed: true)
    t
fn times<P>(a: List<Int>, by: List<Mod<P>>) -> List<Mod<P>>:
    var t = transform(Mod<P>(0), a)
    for i in 0..size: t[i] *= by[i]
    ntt(mut t, inverse: true, bit_reversed: true)
    t

const P1 = 998_244_353
type M1 = Mod<P1>
type M2 = Mod<167_772_161>
type M3 = Mod<469_762_049>

let symbol: List<Int> = [if i == 0: 0 else: (comb.catalan(i - 1) ** 2).v for i in 0..=n + 1]
var cur: List<Int> = [if i == 0: 1 else: 0 for i in 0..=n + 1]
let s1 = transform(M1(0), symbol)
let s2 = transform(M2(0), symbol)
let s3 = transform(M3(0), symbol)
var ans: [0..=n]Z = 0
for k in 0..=n:
    let r1 = times(cur, s1)
    let r2 = times(cur, s2)
    let r3 = times(cur, s3)
    // 47_450_712 is 1 / p1 mod p2 and 115_990_628 is 1 / (p1 p2) mod p3
    for i in 0..=n + 1:
        let x1 = r1[i].v
        let c = x1 + ((r2[i] - M2(x1)) * M2(47_450_712)).v * P1
        let x3 = ((r3[i] - M3(c)) * M3(115_990_628)).v
        cur[i] = (Z(c) + Z(x3) * Z(167_477_612_308_856_833)).v
    ans[k] = Z(cur[n + 1]) * comb.choose(2 * n, k) * comb.inv(k + 1)

fn shift(j: Int) -> Int:
    var tot: Z = 0
    for k in j..=n:
        let term = ans[k] * comb.choose(k, j)
        tot += if (k - j) % 2 == 0: term else: -term
    tot.v

print([shift(j) for j in 0..=n])
