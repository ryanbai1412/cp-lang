//! Polynomial multiplication modulo 998244353 by the number-theoretic transform.
use std.mod.{Mod}

/// Coefficients modulo the NTT prime `998244353 = 119 * 2^23 + 1`.
pub type Z = Mod<998244353>

/// The transform of `a` in place; `a.len` must be a power of two, at most `2^23`.
/// With `invert`, the inverse transform (already divided by the length).
pub fn ntt(mut a: List<Z>, invert: Bool):
    let n = a.len
    assert n & (n - 1) == 0
    // bit-reversal permutation
    var j = 0
    for i in 1..n:
        var bit = n / 2
        while j & bit != 0:
            j ^= bit
            bit /= 2
        j ^= bit
        if i < j:
            let t = a[i]
            a[i] = a[j]
            a[j] = t
    var len = 2
    while len <= n:
        var w = Z(3).pow((998244353 - 1) / len)
        if invert: w = w.inv
        var i = 0
        while i < n:
            var x = Z(1)
            for k in 0..len / 2:
                let (u, v) = (a[i + k], a[i + k + len / 2] * x)
                a[i + k] = u + v
                a[i + k + len / 2] = u - v
                x = x * w
            i += len
        len *= 2
    if invert:
        let inv_n = Z(n).inv
        for i in 0..n: a[i] = a[i] * inv_n

/// The product of two polynomials given by their coefficients, lowest degree first.
/// O((n + m) log(n + m)); short inputs are multiplied directly.
pub fn convolve(a: List<Z>, b: List<Z>) -> List<Z>:
    if a.is_empty or b.is_empty: return []
    let out_len = a.len + b.len - 1
    if min(a.len, b.len) <= 16:
        var out = List.filled(out_len, Z(0))
        for i in 0..a.len:
            for j in 0..b.len: out[i + j] = out[i + j] + a[i] * b[j]
        return out
    var size = 1
    while size < out_len: size *= 2
    var fa = [if i < a.len: a[i] else: Z(0) for i in 0..size]
    var fb = [if i < b.len: b[i] else: Z(0) for i in 0..size]
    ntt(mut fa, false)
    ntt(mut fb, false)
    for i in 0..size: fa[i] = fa[i] * fb[i]
    ntt(mut fa, true)
    [fa[i] for i in 0..out_len]

/// `p` to the power `e`, keeping only the first `keep` coefficients. O(keep log keep log e).
pub fn pow_trunc(p: List<Z>, e: Int, keep: Int) -> List<Z>:
    var acc = [Z(1)]
    var base = [p[i] for i in 0..min(p.len, keep)]
    var k = e
    while k > 0:
        if k % 2 == 1:
            let prod = convolve(acc, base)
            acc = [prod[i] for i in 0..min(prod.len, keep)]
        let sq = convolve(base, base)
        base = [sq[i] for i in 0..min(sq.len, keep)]
        k /= 2
    acc

@test
fn the_transform_round_trips():
    for bits in 0..=6:
        let a = [Z(rand(0, 998244352)) for _ in 0..(1 << bits)]
        var b = [x for x in a]
        ntt(mut b, false)
        ntt(mut b, true)
        assert a == b

@test
fn convolution_matches_the_schoolbook_product():
    for _ in 0..12:
        let a = [Z(rand(0, 998244352)) for _ in 0..rand(0, 45)]
        let b = [Z(rand(0, 998244352)) for _ in 0..rand(0, 45)]
        let got = convolve(a, b)
        assert got.len == (if a.is_empty or b.is_empty: 0 else: a.len + b.len - 1)
        for k in 0..got.len:
            var want = Z(0)
            for i in 0..a.len:
                if 0 <= k - i < b.len: want = want + a[i] * b[k - i]
            assert got[k] == want

@test
fn counting_dice_sums():
    // the number of ways for 20 dice to sum to 70 is a coefficient of (x + ... + x^6)^20
    let die = [Z(0), Z(1), Z(1), Z(1), Z(1), Z(1), Z(1)]
    let ways = pow_trunc(die, 20, 121)
    var dp = [Z(1)]
    for _ in 0..20:
        var next = List.filled(dp.len + 6, Z(0))
        for s in 0..dp.len:
            for face in 1..=6: next[s + face] = next[s + face] + dp[s]
        dp = next
    assert ways.len == 121 and ways[70] == dp[70] and ways[19] == Z(0) and ways[120] == Z(1)
