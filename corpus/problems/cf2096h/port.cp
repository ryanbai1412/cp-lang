// Source: Codeforces 2096H (Wonderful XOR Problem), https://codeforces.com/contest/2096/problem/H
// limits: 2000 ms, 256 MB
// Problem: n intervals [l_i, r_i] in 0..2^m (n <= 2e5, m <= 18); f_x counts sequences with
//   l_i <= a_i <= r_i and XOR x, mod 998244353. Print the XOR over x of (f_x * 2^x mod p).
//   Multitest, sum n <= 2e5, sum 2^m <= 2^18.
// Solution: after a Walsh-Hadamard transform, hat[mask] = prod_i sum_{a in [l_i, r_i]}
//   (-1)^pop(a & mask). For masks with lowest set bit q - 1, each interval's factor is a sign
//   times vr - vl or vr + vl (a triangle wave on the low q bits), picked by the parity of
//   pop(((l ^ r) >> q) & hi). Bucket the two factors by (l ^ r) >> q; a multiplicative butterfly
//   gives the product for every high part hi, and an inverse FWHT recovers f. O((n + 2^m) m).
use std.ntt.{fwht}

type Z = Mod<998_244_353>

// Prefix sum `sum_{a in 0..t} (-1)^{popcount(a & half)}` for `0 <= t < 2 * half`.
fn triangle(t: Int, half: Int) -> Int = if t >= half: 2 * half - t else: t

@!multitest
let n: 1..=200_000 = read()
let m: 1..=18 = read()
let segs: [0..n](0..=262_143, 0..=262_143) = read()
let full = 1 << m

var hat = List.filled(full, Z(0))
hat[0] = 1
for (l, r) in segs: hat[0] *= r - l + 1

for q in 1..=m:
    let half = 1 << (q - 1)
    let low_mask = 2 * half - 1
    let hi_count = 1 << (m - q)
    domain High = 0..hi_count
    var even_prod: [High]Z = 1
    var odd_prod: [High]Z = 1
    var high_xor = 0
    for (l, last) in segs:
        let r = (last + 1) & (full - 1)
        let vl = triangle(l & low_mask, half)
        let vr = triangle(r & low_mask, half)
        high_xor ^= r >> q
        let hx = High((l ^ r) >> q)
        even_prod[hx] *= vr - vl
        odd_prod[hx] *= vr + vl
    // Multiplicative butterfly: combine `even_prod` / `odd_prod` across high bits.
    for bit_idx in 0..m - q:
        let bit = 1 << bit_idx
        for base in (0..hi_count).step(2 * bit):
            for off in 0..bit:
                let a = High(base + off)
                let b = High(base + off + bit)
                (even_prod[a], odd_prod[a], even_prod[b], odd_prod[b]) = (
                    even_prod[a] * even_prod[b],
                    odd_prod[a] * odd_prod[b],
                    even_prod[a] * odd_prod[b],
                    odd_prod[a] * even_prod[b],
                )
    for hi in High:
        let val = even_prod[hi]
        hat[(hi.ord << q) + half] = if popcount(high_xor & hi.ord) % 2 == 1: -val else: val

fwht(mut hat, inverse: true)

var checksum = 0
var pow2: Z = 1
for ways in hat:
    checksum ^= (ways * pow2).v
    pow2 *= 2
print(checksum)
