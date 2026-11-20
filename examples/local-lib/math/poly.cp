//! Polynomials and formal power series modulo 998244353, on the number-theoretic transform.
//!
//! A polynomial is its `List` of coefficients, lowest degree first. The
//! power-series functions take a `keep` and return exactly the first `keep`
//! coefficients of the answer, so `exp_trunc(log_trunc(p, n), n)` is `p`
//! modulo `x^n`.
use std.mod.{Mod}

/// Coefficients modulo the NTT prime `998244353 = 119 * 2^23 + 1`.
pub type Z = Mod<998244353>

/// The transform of `a` in place; `a.len` must be a power of two, at most `2^23`.
/// With `invert`, the inverse transform (already divided by the length).
pub fn ntt(mut a: List<Z>, invert: Bool):
    ntt_p(mut a, 3, invert)

/// The transform modulo any prime `P` with primitive root `root`; `a.len` must divide `P - 1`.
fn ntt_p<P>(mut a: List<Mod<P>>, root: Int, invert: Bool):
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
    // roots[half + k] = w^k for the root of unity `w` of order `2 * half`
    var roots = List.filled(n, Mod<P>(1))
    var half = 1
    while half < n:
        let w = Mod<P>(root).pow((P - 1) / (2 * half))
        for k in 1..half: roots[half + k] = roots[half + k - 1] * w
        var i = 0
        while i < n:
            for k in 0..half:
                let (u, v) = (a[i + k], a[i + k + half] * roots[half + k])
                a[i + k] = u + v
                a[i + k + half] = u - v
            i += 2 * half
        half *= 2
    if invert:
        // the inverse transform is the forward one read backwards, over n
        let inv_n = Mod<P>(n).inv
        a[0] = a[0] * inv_n
        for i in 1..=n / 2:
            let (x, y) = (a[i] * inv_n, a[n - i] * inv_n)
            a[i] = y
            a[n - i] = x

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

// ---- formal power series: every result is the series modulo `x^keep`

/// The first `keep` coefficients of `p`, padded with zeros when `p` is shorter.
fn head(p: List<Z>, keep: Int) -> List<Z> = [if i < p.len: p[i] else: Z(0) for i in 0..keep]

/// The derivative of `p`: one coefficient shorter, and `[]` for a constant. O(n).
pub fn derivative(p: List<Z>) -> List<Z> = [p[i] * Z(i) for i in 1..p.len]

/// The integral of `p` with constant term 0: one coefficient longer. O(n).
pub fn integral(p: List<Z>) -> List<Z>:
    // inv[i] = 1 / i, from `P = (P / i) * i + P % i`
    var inv = [Z(0), Z(1)]
    for i in 2..=p.len: inv.push(-Z(998244353 / i) * inv[998244353 % i])
    [if i == 0: Z(0) else: p[i - 1] * inv[i] for i in 0..=p.len]

/// `1 / p` modulo `x^keep`; `p[0]` must not be zero. O(keep log keep).
pub fn inv_trunc(p: List<Z>, keep: Int) -> List<Z>:
    assert not p.is_empty and p[0].v != 0
    // Newton's method: each step doubles the number of correct coefficients, g <- g * (2 - p * g)
    var g = [p[0].inv]
    while g.len < keep:
        let size = 2 * g.len
        var rest = head(convolve(head(p, size), g), size)
        for i in 0..size: rest[i] = -rest[i]
        rest[0] = rest[0] + Z(2)
        g = head(convolve(g, rest), size)
    head(g, keep)

/// `ln p` modulo `x^keep`, as the integral of `p' / p`; `p[0]` must be 1. O(keep log keep).
pub fn log_trunc(p: List<Z>, keep: Int) -> List<Z>:
    assert not p.is_empty and p[0].v == 1
    if keep <= 0: return []
    integral(head(convolve(derivative(head(p, keep)), inv_trunc(p, keep)), keep - 1))

/// `e^p` modulo `x^keep`; `p[0]` must be 0 (an empty `p` is the zero series). O(keep log keep).
pub fn exp_trunc(p: List<Z>, keep: Int) -> List<Z>:
    assert p.is_empty or p[0].v == 0
    // Newton's method: g <- g * (1 + p - ln g)
    var g = [Z(1)]
    while g.len < keep:
        let size = 2 * g.len
        let lg = log_trunc(g, size)
        var rest = head(p, size)
        for i in 0..size: rest[i] = rest[i] - lg[i]
        rest[0] = rest[0] + Z(1)
        g = head(convolve(g, rest), size)
    head(g, keep)

/// `p` to the power `e >= 0` modulo `x^keep`, with `p^0 = 1` even for the zero series.
/// O(keep log keep) whatever `e` is, by `p^e = e^(e ln p)` after factoring out the lowest term.
pub fn pow_trunc(p: List<Z>, e: Int, keep: Int) -> List<Z>:
    assert e >= 0
    if e == 0: return head([Z(1)], keep)
    // p = c x^low * q with q[0] = 1, so p^e = c^e x^(low * e) * q^e
    var low = 0
    while low < p.len and p[low].v == 0: low += 1
    let shift = if low > 0 and e >= keep: keep else: low * e
    if low == p.len or shift >= keep: return head([], keep)
    let lead_inv = p[low].inv
    let q = [p[i] * lead_inv for i in low..p.len]
    let scaled = [x * Z(e) for x in log_trunc(q, keep - shift)]
    let lead = p[low].pow(e)
    let power = exp_trunc(scaled, keep - shift)
    [if i < shift: Z(0) else: power[i - shift] * lead for i in 0..keep]

// ---- arbitrary-modulus 3-prime MTT convolution & Lagrange interpolation

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

// ---- power series against O(n^2) references

fn random_series(len: Int) -> List<Z> = [Z(rand(0, 998244352)) for _ in 0..len]

/// A series with a usable constant term: `first`, then `len - 1` random coefficients.
fn random_from(first: Z, len: Int) -> List<Z> = [if i == 0: first else: Z(rand(0, 998244352)) for i in 0..len]

fn coef(p: List<Z>, i: Int) -> Z = if i < p.len: p[i] else: Z(0)

/// `(p.len, keep)` pairs around the powers of two, with `p` shorter than, as long as and longer than `keep`.
fn shapes() -> List<(Int, Int)> = [(1, 0), (1, 1), (1, 9), (2, 16), (3, 17), (8, 8), (17, 32), (31, 33), (33, 20), (40, 40)]

fn slow_mul(a: List<Z>, b: List<Z>, keep: Int) -> List<Z>:
    var out = List.filled(keep, Z(0))
    for i in 0..min(a.len, keep):
        for j in 0..min(b.len, keep - i): out[i + j] = out[i + j] + a[i] * b[j]
    out

/// From `p * g = 1`: `p[0] * g[n] = -sum(p[i] * g[n - i] for i in 1..=n)`.
fn slow_inv(p: List<Z>, keep: Int) -> List<Z>:
    var g = List.filled(keep, Z(0))
    for n in 0..keep:
        var acc = if n == 0: Z(1) else: Z(0)
        for i in 1..=n: acc = acc - coef(p, i) * g[n - i]
        g[n] = acc / p[0]
    g

/// From `p * l' = p'`: `n * l[n] = n * p[n] - sum(i * l[i] * p[n - i] for i in 1..n)`.
fn slow_log(p: List<Z>, keep: Int) -> List<Z>:
    var l = List.filled(keep, Z(0))
    for n in 1..keep:
        var acc = Z(n) * coef(p, n)
        for i in 1..n: acc = acc - Z(i) * l[i] * coef(p, n - i)
        l[n] = acc / Z(n)
    l

/// From `g' = p' * g`: `n * g[n] = sum(i * p[i] * g[n - i] for i in 1..=n)`.
fn slow_exp(p: List<Z>, keep: Int) -> List<Z>:
    var g = List.filled(keep, Z(0))
    for n in 0..keep:
        var acc = if n == 0: Z(1) else: Z(0)
        for i in 1..=n: acc = acc + Z(i) * coef(p, i) * g[n - i]
        g[n] = if n == 0: acc else: acc / Z(n)
    g

fn slow_pow(p: List<Z>, e: Int, keep: Int) -> List<Z>:
    var acc = slow_mul([Z(1)], [Z(1)], keep)
    for _ in 0..e: acc = slow_mul(acc, p, keep)
    acc

@test
fn derivative_and_integral_undo_each_other():
    assert derivative([]) == [] and derivative([Z(7)]) == []
    assert derivative([Z(5), Z(3), Z(2)]) == [Z(3), Z(4)]
    assert integral([]) == [Z(0)]
    assert integral([Z(3), Z(4)]) == [Z(0), Z(3), Z(2)]
    for len in [0, 1, 2, 5, 16, 33]:
        let p = random_series(len)
        assert derivative(integral(p)) == p
        // integrating a derivative loses only the constant term
        if len > 0: assert integral(derivative(p)) == [if i == 0: Z(0) else: p[i] for i in 0..len]

@test
fn inverse_matches_the_recurrence():
    // 1 / (1 - x) = 1 + x + x^2 + ...
    assert inv_trunc([Z(1), Z(-1)], 40) == List.filled(40, Z(1))
    assert inv_trunc([Z(2)], 3) == [Z(2).inv, Z(0), Z(0)] and inv_trunc([Z(2)], 0) == []
    for (len, keep) in shapes():
        let p = random_from(Z(rand(1, 998244352)), len)
        let got = inv_trunc(p, keep)
        assert got == slow_inv(p, keep)
        assert slow_mul(p, got, keep) == slow_mul([Z(1)], [Z(1)], keep)
        // zero coefficients on top of `p` change nothing
        assert inv_trunc([coef(p, i) for i in 0..len + 5], keep) == got

@test
fn log_and_exp_match_their_recurrences():
    // ln(1 / (1 - x)) = x + x^2 / 2 + x^3 / 3 + ..., and e^x = sum x^k / k!
    let harmonic = log_trunc(List.filled(40, Z(1)), 40)
    let e_x = exp_trunc([Z(0), Z(1)], 40)
    var fact = Z(1)
    for k in 1..40:
        fact = fact * Z(k)
        assert harmonic[k] * Z(k) == Z(1) and e_x[k] * fact == Z(1)
    assert harmonic[0] == Z(0) and e_x[0] == Z(1)
    // the zero series, as `[]` or as zeros, and `keep = 0`
    assert exp_trunc([], 5) == [Z(1), Z(0), Z(0), Z(0), Z(0)]
    assert exp_trunc([Z(0), Z(0)], 3) == [Z(1), Z(0), Z(0)]
    assert log_trunc([Z(1)], 4) == [Z(0), Z(0), Z(0), Z(0)]
    assert log_trunc([Z(1), Z(5)], 0) == [] and exp_trunc([Z(0), Z(5)], 0) == []
    for (len, keep) in shapes():
        let p = random_from(Z(1), len)
        let lg = log_trunc(p, keep)
        assert lg == slow_log(p, keep)
        assert exp_trunc(lg, keep) == [coef(p, i) for i in 0..keep]
        let q = random_from(Z(0), len)
        let ex = exp_trunc(q, keep)
        assert ex == slow_exp(q, keep)
        if keep > 0: assert log_trunc(ex, keep) == [coef(q, i) for i in 0..keep]

@test
fn power_matches_repeated_multiplication():
    for (len, keep) in shapes():
        for (low, e) in [(0, 0), (0, 1), (0, 5), (1, 2), (2, 0), (3, 7), (9, 3)]:
            // `low` zero coefficients, then a non-zero one
            let p = [if i < low: Z(0) else: Z(rand(1, 998244352)) for i in 0..low + len]
            assert pow_trunc(p, e, keep) == slow_pow(p, e, keep)
    // the zero series: 0^0 = 1, every other power is 0
    assert pow_trunc([], 0, 3) == [Z(1), Z(0), Z(0)] and pow_trunc([], 4, 3) == [Z(0), Z(0), Z(0)]
    assert pow_trunc([Z(0), Z(0)], 0, 2) == [Z(1), Z(0)] and pow_trunc([Z(0), Z(0)], 1, 2) == [Z(0), Z(0)]

@test
fn power_takes_exponents_far_beyond_the_length():
    // (1 + x)^e = sum C(e, k) x^k, built up by C(e, k) = C(e, k - 1) * (e - k + 1) / k
    let e = 1_000_000_000_000_000_000
    let got = pow_trunc([Z(1), Z(1)], e, 20)
    var want = Z(1)
    for k in 0..20:
        assert got[k] == want
        want = want * Z(e - k) / Z(k + 1)
    // (3 x)^e is gone below x^e, without overflowing on the way
    assert pow_trunc([Z(0), Z(3)], e, 20) == List.filled(20, Z(0))
    // the constant term is raised to the true exponent: 3^e, not 3^(e mod 998244353)
    assert pow_trunc([Z(3)], e, 2) == [Z(3).pow(e), Z(0)]
    // x^2 * (2 + x), cubed, starts at 8 x^6
    assert pow_trunc([Z(0), Z(0), Z(2), Z(1)], 3, 9) == [Z(0), Z(0), Z(0), Z(0), Z(0), Z(0), Z(8), Z(12), Z(6)]
