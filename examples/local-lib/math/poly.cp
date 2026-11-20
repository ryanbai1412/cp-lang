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

// ---- arbitrary-modulus 3-prime MTT convolution & Lagrange interpolation

fn ntt_p<P>(mut a: List<Mod<P>>, root: Int, invert: Bool):
    let n = a.len
    assert n & (n - 1) == 0
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
        var w = Mod<P>(root).pow((P - 1) / len)
        if invert: w = w.inv
        var i = 0
        while i < n:
            var x = Mod<P>(1)
            for k in 0..len / 2:
                let (u, v) = (a[i + k], a[i + k + len / 2] * x)
                a[i + k] = u + v
                a[i + k + len / 2] = u - v
                x = x * w
            i += len
        len *= 2
    if invert:
        let inv_n = Mod<P>(n).inv
        for i in 0..n: a[i] = a[i] * inv_n

fn convolve_p<P>(a: List<Int>, b: List<Int>, root: Int) -> List<Int>:
    let out_len = a.len + b.len - 1
    var size = 1
    while size < out_len: size *= 2
    var fa = [if i < a.len: Mod<P>(a[i]) else: Mod<P>(0) for i in 0..size]
    var fb = [if i < b.len: Mod<P>(b[i]) else: Mod<P>(0) for i in 0..size]
    ntt_p(mut fa, root, false)
    ntt_p(mut fb, root, false)
    for i in 0..size: fa[i] = fa[i] * fb[i]
    ntt_p(mut fa, root, true)
    [fa[i].v for i in 0..out_len]

/// Convolution of two integer polynomials modulo any `m` (`1 <= m <= 2e9`), using 3-prime NTT
/// (MTT) and Garner's CRT with zero 64-bit overflow. O((n + m) log(n + m)).
pub fn convolve_mod(a: List<Int>, b: List<Int>, m: Int) -> List<Int>:
    if a.is_empty or b.is_empty: return []
    let out_len = a.len + b.len - 1
    if m == 1: return List.filled(out_len, 0)
    let ra = [x % m for x in a]
    let rb = [x % m for x in b]
    if min(ra.len, rb.len) <= 16:
        var out = List.filled(out_len, 0)
        for i in 0..ra.len:
            for j in 0..rb.len: out[i + j] = (out[i + j] + ra[i] * rb[j]) % m
        return out
    // Three NTT primes whose product exceeds 7.8e25 (> max_n * (10^9+7)^2)
    let p1 = 167_772_161
    let p2 = 469_762_049
    let p3 = 998_244_353
    let c1 = convolve_p::<167772161>(ra, rb, 3)
    let c2 = convolve_p::<469762049>(ra, rb, 3)
    let c3 = convolve_p::<998244353>(ra, rb, 3)
    let inv12 = Mod<469762049>(p1).inv.v
    let inv123 = (Mod<998244353>(p1) * Mod<998244353>(p2)).inv.v
    let p1_m = p1 % m
    let p12_m = p1_m * (p2 % m) % m
    var out = List.filled(out_len, 0)
    for i in 0..out_len:
        let r1 = c1[i]
        let t1 = (c2[i] - r1) % p2 * inv12 % p2
        let cur3 = (r1 + t1 * p1 % p3) % p3
        let t2 = (c3[i] - cur3) % p3 * inv123 % p3
        out[i] = (r1 % m + t1 % m * p1_m % m + t2 % m * p12_m % m) % m
    out

/// Evaluate polynomial `p` (lowest degree first) at `x` by Horner's rule. O(n).
pub fn eval_poly<P>(p: List<Mod<P>>, x: Mod<P>) -> Mod<P>:
    var acc = Mod<P>(0)
    for i in (0..p.len).rev: acc = acc * x + p[i]
    acc

/// Unique polynomial of degree `< pts.len` passing through every `(x, y)` in `pts`
/// (all `x` values must be distinct). Returns coefficients lowest degree first. O(n^2).
pub fn interpolate<P>(pts: List<(Mod<P>, Mod<P>)>) -> List<Mod<P>>:
    let n = pts.len
    if n == 0: return []
    // Build full product Q(x) = prod (x - x_i)
    var q = [Mod<P>(1)]
    for (xi, _) in pts:
        var nxt = List.filled(q.len + 1, Mod<P>(0))
        for j in 0..q.len:
            nxt[j] = nxt[j] - q[j] * xi
            nxt[j + 1] = nxt[j + 1] + q[j]
        q = nxt
    var res = List.filled(n, Mod<P>(0))
    for i in 0..n:
        let (xi, yi) = pts[i]
        var denom = Mod<P>(1)
        for j in 0..n:
            if i != j: denom = denom * (xi - pts[j].0)
        let coeff = yi * denom.inv
        // Synthetic division of Q(x) by (x - xi), highest degree down
        var rem = q[n]
        for j in (0..n).rev:
            res[j] = res[j] + rem * coeff
            if j > 0: rem = q[j] + rem * xi
    res

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

@test
fn mtt_arbitrary_modulus_and_interpolation():
    // 1. Arbitrary prime & composite moduli (including 10^9+7) match schoolbook product
    for m in [1_000_000_007, 1_000_000_009, 1_000_000_000]:
        for _ in 0..4:
            let a = [rand(0, m - 1) for _ in 0..rand(18, 30)]
            let b = [rand(0, m - 1) for _ in 0..rand(18, 30)]
            let got = convolve_mod(a, b, m)
            for k in 0..got.len:
                var want = 0
                for i in 0..a.len:
                    if 0 <= k - i < b.len: want = (want + a[i] * b[k - i]) % m
                assert got[k] == want
    // 2. Lagrange interpolation recovers arbitrary polynomials over 10^9+7
    type M = Mod<1000000007>
    for deg in 0..=8:
        let poly = [M(rand(0, 1_000_000_006)) for _ in 0..=deg]
        let pts = [(M(i * 3 - 5), eval_poly(poly, M(i * 3 - 5))) for i in 0..=deg]
        let got = interpolate(pts)
        assert got == poly
        for (x, y) in pts: assert eval_poly(got, x) == y
