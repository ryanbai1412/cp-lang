//! Formal power series operations over NTT-friendly `Mod<P>`.
//!
//! Series are plain `List<Mod<P>>` (lowest degree first: `a[k]` is `[x^k] a(x)`),
//! matching `std.ntt.convolve`. Every truncated operation takes an explicit output
//! length `n >= 0` (computing modulo `x^n`, returning length `n`; `n == 0` returns `[]`).
use std.math.{mod_sqrt}
use std.mod.{Mod}
use std.ntt.{convolve, ntt}

/// Formal derivative `a'(x)`. Length `max(0, a.len - 1)`. O(a.len).
pub fn deriv<P>(a: List<Mod<P>>) -> List<Mod<P>>:
    let m = a.len
    if m <= 1: return []
    var out = List.filled(m - 1, Mod<P>(0))
    unchecked:
        for i in 1..m: out[i - 1] = a[i] * Mod<P>(i)
    out

/// Formal antiderivative with constant term 0. Length `a.len + 1` (or `[]` if empty).
/// Uses linear-time modular inverses `1..=a.len`. O(a.len).
pub fn integral<P>(a: List<Mod<P>>) -> List<Mod<P>>:
    let m = a.len
    if m == 0: return []
    assert m < P, "poly.integral requires a.len < P"
    var invs = List.filled(m + 1, Mod<P>(0))
    var out = List.filled(m + 1, Mod<P>(0))
    unchecked:
        invs[1] = Mod<P>(1)
        for i in 2..=m: invs[i] = -Mod<P>(P / i) * invs[P % i]
        for i in 0..m: out[i + 1] = a[i] * invs[i + 1]
    out

/// Multiplicative inverse `a(x)^(-1) mod x^n` via half-size Newton doubling.
/// Requires `n >= 0`, and when `n > 0`, `a.len > 0` and `a[0] != 0`. O(n log n).
pub fn inv<P>(a: List<Mod<P>>, n: Int) -> List<Mod<P>>:
    assert n >= 0, "poly.inv requires n >= 0"
    if n == 0: return []
    assert a.len > 0 and a[0].v != 0, "poly.inv requires a[0] != 0"
    var g: List<Mod<P>> = [a[0].inv]
    while g.len < n:
        let m = g.len
        let lim = m * 2
        let take = min(a.len, lim)
        var f = List.filled(lim, Mod<P>(0))
        var gn = List.filled(lim, Mod<P>(0))
        unchecked:
            for i in 0..take: f[i] = a[i]
            for i in 0..m: gn[i] = g[i]
        ntt(mut f)
        ntt(mut gn)
        unchecked:
            for i in 0..lim: f[i] = f[i] * gn[i]
        ntt(mut f, inverse: true)
        unchecked:
            for i in 0..m: f[i] = Mod<P>(0)
        ntt(mut f)
        unchecked:
            for i in 0..lim: f[i] = f[i] * gn[i]
        ntt(mut f, inverse: true)
        unchecked:
            for i in m..lim: g.push(-f[i])
    if g.len > n: g.truncate(n)
    g

/// Formal logarithm `ln(a(x)) mod x^n = integral(a'(x) / a(x)) mod x^n`.
/// Requires `n >= 0`, and when `n > 0`, `a.len > 0` and `a[0] == 1`. O(n log n).
pub fn log<P>(a: List<Mod<P>>, n: Int) -> List<Mod<P>>:
    assert n >= 0, "poly.log requires n >= 0"
    if n == 0: return []
    assert a.len > 0 and a[0].v == 1, "poly.log requires a[0] == 1"
    if n == 1: return [Mod<P>(0)]
    let take = min(a.len, n)
    var da = List.filled(if take > 1: take - 1 else: 0, Mod<P>(0))
    unchecked:
        for i in 1..take: da[i - 1] = a[i] * Mod<P>(i)
    let ia = inv(a, n - 1)
    var q = convolve(da, ia)
    if q.len > n - 1: q.truncate(n - 1)
    while q.len < n - 1: q.push(Mod<P>(0))
    integral(q)

/// Formal exponential `exp(a(x)) mod x^n` via Newton doubling.
/// Requires `n >= 0`, and when `n > 0`, `a.is_empty` or `a[0] == 0`. O(n log n).
pub fn exp<P>(a: List<Mod<P>>, n: Int) -> List<Mod<P>>:
    assert n >= 0, "poly.exp requires n >= 0"
    if n == 0: return []
    assert a.is_empty or a[0].v == 0, "poly.exp requires a[0] == 0"
    var g: List<Mod<P>> = [Mod<P>(1)]
    var m = 1
    while m < n:
        let lim = m * 2
        let lg = log(g, lim)
        var hi = List.filled(m, Mod<P>(0))
        unchecked:
            for i in 0..m:
                let ai = if m + i < a.len: a[m + i] else: Mod<P>(0)
                hi[i] = ai - lg[m + i]
        let delta = convolve(g, hi)
        unchecked:
            for i in 0..m: g.push(if i < delta.len: delta[i] else: Mod<P>(0))
        m = lim
    if g.len > n: g.truncate(n)
    g

/// Formal power `a(x)^k mod x^n` for `k >= 0`, handling leading zeros. O(n log n).
pub fn pow<P>(a: List<Mod<P>>, k: Int, n: Int) -> List<Mod<P>>:
    assert n >= 0 and k >= 0, "poly.pow requires n >= 0 and k >= 0"
    if n == 0: return []
    if k == 0:
        var out = List.filled(n, Mod<P>(0))
        out[0] = Mod<P>(1)
        return out
    var d = 0
    let cap = min(a.len, n)
    while d < cap and a[d].v == 0: d += 1
    if d == cap or (d > 0 and k > (n - 1) / d):
        return List.filled(n, Mod<P>(0))
    let shift = k * d
    let m = n - shift
    let c = a[d]
    let inv_c = c.inv
    let take = min(a.len - d, m)
    var norm = List.filled(take, Mod<P>(0))
    unchecked:
        for i in 0..take: norm[i] = a[d + i] * inv_c
    var lg = log(norm, m)
    let km = Mod<P>(k)
    unchecked:
        for i in 0..m: lg[i] = lg[i] * km
    let ex = exp(lg, m)
    let ck = c.pow(k)
    var out = List.filled(n, Mod<P>(0))
    unchecked:
        for i in 0..m: out[shift + i] = ex[i] * ck
    out

/// Formal square root `g(x)` with `g(x)^2 == a(x) (mod x^n)`, or `none` if none exists mod `P`.
/// Requires `n >= 0`. O(n log n + log^2 P).
pub fn sqrt<P>(a: List<Mod<P>>, n: Int) -> ?List<Mod<P>>:
    assert n >= 0, "poly.sqrt requires n >= 0"
    if n == 0: return some([])
    var d = 0
    let cap = min(a.len, n)
    while d < cap and a[d].v == 0: d += 1
    if d == cap: return some(List.filled(n, Mod<P>(0)))
    if d % 2 != 0: return none
    let r0 = if let r = mod_sqrt(a[d].v, P): r else: return none
    let shift = d / 2
    let m = n - shift
    let h_len = min(a.len - d, m)
    var h = List.filled(h_len, Mod<P>(0))
    unchecked:
        for i in 0..h_len: h[i] = a[d + i]
    let inv2 = Mod<P>(2).inv
    var g: List<Mod<P>> = [Mod<P>(r0)]
    var cur = 1
    while cur < m:
        let lim = cur * 2
        var g2 = List.filled(lim, Mod<P>(0))
        unchecked:
            for i in 0..cur: g2[i] = g[i]
        ntt(mut g2)
        unchecked:
            for i in 0..lim: g2[i] = g2[i] * g2[i]
        ntt(mut g2, inverse: true)
        var diff = List.filled(cur, Mod<P>(0))
        unchecked:
            for i in 0..cur:
                let hi = if cur + i < h_len: h[cur + i] else: Mod<P>(0)
                diff[i] = (hi - g2[cur + i]) * inv2
        let ig = inv(g, cur)
        let delta = convolve(diff, ig)
        unchecked:
            for i in 0..cur: g.push(if i < delta.len: delta[i] else: Mod<P>(0))
        cur = lim
    var out = List.filled(n, Mod<P>(0))
    unchecked:
        for i in 0..m: out[shift + i] = g[i]
    some(out)

@test
fn deriv_and_integral_laws():
    type Z = Mod<998_244_353>
    let a: List<Z> = [Z(3), Z(5), Z(7), Z(11)]
    let da = deriv(a)
    assert da == [Z(5), Z(14), Z(33)]
    assert deriv(integral(a)) == a
    let empty: List<Z> = []
    assert deriv(empty).is_empty
    assert integral(empty).is_empty

@test
fn inv_log_exp_laws():
    type Z = Mod<998_244_353>
    let a = [Z(1 + (i * 7 + 3) % 19) for i in 0..17]
    let ia = inv(a, 17)
    let prod = convolve(a, ia)
    assert prod[0] == Z(1)
    for i in 1..17: assert prod[i] == Z(0)
    assert inv(a, 0).is_empty

    let b = [if i == 0: Z(1) else: Z((i * 5 + 2) % 23) for i in 0..15]
    let lb = log(b, 15)
    assert lb[0] == Z(0)
    assert exp(lb, 15) == b
    let c = [if i == 0: Z(0) else: Z((i * 3 + 1) % 17) for i in 0..15]
    assert log(exp(c, 15), 15) == c

@test
fn pow_and_sqrt_laws():
    type Z = Mod<998_244_353>
    let a: List<Z> = [Z(0), Z(2), Z(3), Z(5)]
    let p3 = pow(a, 3, 10)
    var naive: List<Z> = [Z(1)]
    for _ in 0..3:
        naive = convolve(naive, a)
        if naive.len > 10: naive.truncate(10)
    while naive.len < 10: naive.push(Z(0))
    assert p3 == naive
    assert pow(a, 0, 4) == [Z(1), Z(0), Z(0), Z(0)]
    assert pow(a, 1_000_000_000_000, 6) == List.filled(6, Z(0))

    let s_in: List<Z> = [Z(0), Z(0), Z(4), Z(12), Z(9), Z(5), Z(7)]
    let g = sqrt(s_in, 7)!
    let gg = convolve(g, g)
    for i in 0..7: assert gg[i] == s_in[i]
    let odd_zero: List<Z> = [Z(0), Z(4), Z(1)]
    assert sqrt(odd_zero, 3) == none
    let non_res: List<Z> = [Z(3), Z(1)]
    assert sqrt(non_res, 2) == none
    assert sqrt(List.filled(5, Z(0)), 5)! == List.filled(5, Z(0))
