// Codeforces 2096 H: Wonderful XOR Sum.
// Given `n` intervals `[l_j, r_j]` in `0..2^m`, we need `ways[x]` = number of
// tuples with XOR `x`, then output `XOR_x ((ways[x] * 2^x) mod P)`.
// Under the Walsh-Hadamard transform (WHT), XOR convolution becomes pointwise
// multiplication: `hat[mask] = prod_j sum_{a in [l_j, r_j]} (-1)^{pop(a & mask)}`.
// Group masks by their lowest set bit `q - 1` (`q in 1..=m`), writing
// `mask = (hi << q) + half` with `half = 1 << (q - 1)`.
// On the low `q` bits, `sum_{a < t} (-1)^{pop(a & half)}` is the triangle wave
// `triangle(t, half) = min(t, 2 * half - t)`.
// For each interval `[l, r)` (`r = (last + 1) mod 2^m`), the contribution at
// `hi` has base sign `(-1)^{pop((r >> q) & hi)}` (folded into `high_xor`) and
// factor `vr - vl` if `pop(hx & hi)` is even (`even_prod`) or `vr + vl` if odd
// (`odd_prod`), where `hx = (l ^ r) >> q`. A multiplicative butterfly over the
// `m - q` high bits evaluates the product for every `hi`, and an inverse WHT
// recovers `ways`.
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
var coeff = Z(1)
for ways in hat:
    checksum ^= (ways * coeff).v
    coeff *= 2
print(checksum)
