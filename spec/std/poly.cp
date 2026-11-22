//! Formal power series operations over NTT-friendly `Mod<P>`.
//!
//! Series are plain `List<Mod<P>>` (lowest degree first: `a[k]` is `[x^k] a(x)`),
//! matching `std.ntt.convolve`. Every truncated operation takes an explicit output
//! length `n >= 0` (computing modulo `x^n`, returning length `n`; `n == 0` returns `[]`).
use std.math.{mod_sqrt}
use std.mod.{Mod}
use std.ntt.{ntt}

fn times<P>(a: List<Mod<P>>, b: List<Mod<P>>) -> List<Mod<P>>:
    if a.is_empty or b.is_empty: return []
    let na = a.len
    let nb = b.len
    let n_out = na + nb - 1
    if min(na, nb) <= 60:
        var out = List.filled(n_out, Mod<P>(0))
        unchecked:
            for i in 0..na:
                let ai = a[i]
                if ai.v == 0: continue
                for j in 0..nb: out[i + j] = out[i + j] + ai * b[j]
        return out
    let need = bit_ceil(n_out)
    assert (P - 1) % need == 0, "modulus p - 1 must be divisible by n"
    var fa = List.filled(need, Mod<P>(0))
    var fb = List.filled(need, Mod<P>(0))
    unchecked:
        for i in 0..na: fa[i] = a[i]
        for i in 0..nb: fb[i] = b[i]
    ntt(mut fa, bit_reversed: true)
    ntt(mut fb, bit_reversed: true)
    unchecked:
        for i in 0..need: fa[i] = fa[i] * fb[i]
    ntt(mut fa, inverse: true, bit_reversed: true)
    if fa.len > n_out: fa.truncate(n_out)
    fa

/// The formal derivative `a'(x)`: coefficient `k` is `(k + 1) * a[k + 1]`. Length
/// `max(0, a.len - 1)`, trailing zeros kept.
///
/// cost: O(a.len)
/// empty: `[]`
/// order: lowest degree first
/// traps: never
/// example: `type Z = Mod<998244353>; let a: List<Z> = [1, 2, 3]; deriv(a) == [Z(2), Z(6)]`
/// example: `type Z = Mod<998244353>; let a: List<Z> = [5]; deriv(a) == []`
pub fn deriv<P>(a: List<Mod<P>>) -> List<Mod<P>>:
    let m = a.len
    if m <= 1: return []
    var out = List.filled(m - 1, Mod<P>(0))
    unchecked:
        for i in 1..m: out[i - 1] = a[i] * Mod<P>(i)
    out

/// The formal antiderivative with constant term 0: coefficient `k + 1` is `a[k] / (k + 1)`. Length
/// `a.len + 1` (or `[]` if empty). Uses linear-time modular inverses `1..=a.len`.
///
/// cost: O(a.len)
/// empty: `[]`
/// order: lowest degree first
/// traps: when `a.len >= P` (`a.len` has no inverse)
/// example: `type Z = Mod<998244353>; let a: List<Z> = [1, 2, 3]; integral(a) == [Z(0), Z(1), Z(1), Z(1)]`
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

/// The multiplicative inverse `a(x)^(-1) mod x^n` via half-size Newton doubling: the length-`n`
/// `b` with `a * b == 1 (mod x^n)`. Requires `n >= 0`, and when `n > 0`, `a.len > 0` and
/// `a[0] != 0`. Coefficients of `a` at or above `x^n` are ignored.
///
/// cost: O(n log n)
/// empty: `[]` when `n == 0`, whatever `a` is
/// order: lowest degree first
/// traps: when `n < 0`; when `n > 0` and `a` is empty or `a[0] == 0`; when a transform length
/// exceeds what `P` supports
/// example: `type Z = Mod<998244353>; let a: List<Z> = [1, -1]; inv(a, 4) == [Z(1), Z(1), Z(1), Z(1)]`
/// example: `type Z = Mod<998244353>; let a: List<Z> = [2]; inv(a, 2) == [Z(1) / Z(2), Z(0)]`
pub fn inv<P>(a: List<Mod<P>>, n: Int) -> List<Mod<P>>:
    assert n >= 0, "poly.inv() requires n >= 0"
    if n == 0: return []
    assert a.len > 0 and a[0].v != 0, "poly.inv() requires a[0] != 0"
    var g: List<Mod<P>> = [a[0].inv()]
    while g.len < n:
        let m = g.len
        let lim = m * 2
        let take = min(a.len, lim)
        var f = List.filled(lim, Mod<P>(0))
        var gn = List.filled(lim, Mod<P>(0))
        unchecked:
            for i in 0..take: f[i] = a[i]
            for i in 0..m: gn[i] = g[i]
        ntt(mut f, bit_reversed: true)
        ntt(mut gn, bit_reversed: true)
        unchecked:
            for i in 0..lim: f[i] = f[i] * gn[i]
        ntt(mut f, inverse: true, bit_reversed: true)
        unchecked:
            for i in 0..m: f[i] = Mod<P>(0)
        ntt(mut f, bit_reversed: true)
        unchecked:
            for i in 0..lim: f[i] = f[i] * gn[i]
        ntt(mut f, inverse: true, bit_reversed: true)
        unchecked:
            for i in m..lim: g.push(-f[i])
    if g.len > n: g.truncate(n)
    g

/// The formal logarithm `ln(a(x)) mod x^n = integral(a'(x) / a(x)) mod x^n`, with constant term 0.
/// Requires `n >= 0`, and when `n > 0`, `a.len > 0` and `a[0] == 1`.
///
/// cost: O(n log n)
/// empty: `[]` when `n == 0`
/// order: lowest degree first
/// traps: when `n < 0`; when `n > 0` and `a` is empty or `a[0] != 1`; when `n > P` (the
/// integral needs inverses of `1..n`); when a transform length exceeds what `P` supports
/// example: `type Z = Mod<998244353>; let a: List<Z> = [1, 2, 1]; log(a, 3) == [Z(0), Z(2), Z(-1)]`
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
    var q = times(da, ia)
    if q.len > n - 1: q.truncate(n - 1)
    while q.len < n - 1: q.push(Mod<P>(0))
    integral(q)

/// The formal exponential `exp(a(x)) mod x^n` via Newton doubling, with constant term 1.
/// Requires `n >= 0`, and when `n > 0`, `a.is_empty` or `a[0] == 0`.
///
/// cost: O(n log n)
/// empty: `[]` when `n == 0`; `exp([], n)` is `[1, 0, ...]`
/// order: lowest degree first
/// traps: when `n < 0`; when `n > 0` and `a[0] != 0`; when `n > P`; when a transform length
/// exceeds what `P` supports
/// example: `type Z = Mod<998244353>; let a: List<Z> = [0, 2]; exp(a, 3) == [Z(1), Z(2), Z(2)]`
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
        let delta = times(g, hi)
        unchecked:
            for i in 0..m: g.push(if i < delta.len: delta[i] else: Mod<P>(0))
        m = lim
    if g.len > n: g.truncate(n)
    g

/// The formal power `a(x)^k mod x^n` (supports `k < 0` when `a[0] != 0`).
///
/// `k == 0` gives `[1, 0, ...]` even when `a` is zero (as repeated `convolve` does). For `k > 0`
/// leading zero coefficients are allowed: `t` leading zeros become `t * k`.
///
/// cost: O(n log n)
/// empty: `[]` when `n == 0`
/// order: lowest degree first
/// traps: when `n < 0`; when `k < 0` and `a` is empty or `a[0] == 0`; when a transform length
/// exceeds what `P` supports
/// example: `type Z = Mod<998244353>; let a: List<Z> = [1, 1]; pow(a, 2, 4) == [Z(1), Z(2), Z(1), Z(0)]`
/// example: `type Z = Mod<998244353>; let a: List<Z> = [1, 1]; pow(a, -1, 3) == [Z(1), Z(-1), Z(1)]`
/// example: `type Z = Mod<998244353>; let a: List<Z> = [0, 1]; pow(a, 2, 3) == [Z(0), Z(0), Z(1)]`
pub fn pow<P>(a: List<Mod<P>>, k: Int, n: Int) -> List<Mod<P>>:
    assert n >= 0, "poly.pow requires n >= 0"
    if n == 0: return []
    if k == 0:
        var out = List.filled(n, Mod<P>(0))
        out[0] = Mod<P>(1)
        return out
    var d = 0
    let cap = min(a.len, n)
    while d < cap and a[d].v == 0: d += 1
    if k < 0:
        assert a.len > 0 and d == 0, "poly.pow with k < 0 requires a[0] != 0"
    elif d == cap or (d > 0 and k > (n - 1) / d):
        return List.filled(n, Mod<P>(0))
    let shift = k * d
    let m = n - shift
    let c = a[d]
    let inv_c = c.inv()
    let take = min(a.len - d, m)
    var norm = List.filled(take, Mod<P>(0))
    unchecked:
        for i in 0..take: norm[i] = a[d + i] * inv_c
    var lg = log(norm, m)
    let km = Mod<P>(k)
    unchecked:
        for i in 0..m: lg[i] = lg[i] * km
    let ex = exp(lg, m)
    let ck = c ** k
    var out = List.filled(n, Mod<P>(0))
    unchecked:
        for i in 0..m: out[shift + i] = ex[i] * ck
    out

/// The formal square root `g(x)` with `g(x)^2 == a(x) (mod x^n)`, or `None` if None exists mod `P`.
/// Requires `n >= 0`. O(n log n + log^2 P).
///
/// Of the two roots, the one returned has its lowest nonzero coefficient equal to the smaller
/// square root of `a`'s lowest nonzero coefficient (as an integer in `0..P`). `None` when that
/// coefficient is a non-residue or `a` has an odd number of leading zeros below `x^n`; an `a` that
/// is zero below `x^n` gives all zeros.
///
/// cost: O(n log n + log^2 P)
/// empty: `Some([])` when `n == 0`
/// order: lowest degree first
/// traps: when `n < 0`; when a transform length exceeds what `P` supports
/// example: `type Z = Mod<998244353>; let a: List<Z> = [1, 2, 1]; sqrt(a, 3) == Some([Z(1), Z(1), Z(0)])`
/// example: `type Z = Mod<998244353>; let a: List<Z> = [0, 0, 4]; sqrt(a, 3) == Some([Z(0), Z(2), Z(0)])`
/// example: `type Z = Mod<998244353>; let a: List<Z> = [0, 1]; sqrt(a, 2) == None`
/// example: `type Z = Mod<998244353>; let a: List<Z> = [3]; sqrt(a, 1) == None`
pub fn sqrt<P>(a: List<Mod<P>>, n: Int) -> ?List<Mod<P>>:
    assert n >= 0, "poly.sqrt requires n >= 0"
    if n == 0: return Some([])
    var d = 0
    let cap = min(a.len, n)
    while d < cap and a[d].v == 0: d += 1
    if d == cap: return Some(List.filled(n, Mod<P>(0)))
    if d % 2 != 0: return None
    let r0 = if let r = mod_sqrt(a[d].v, P): r else: return None
    let shift = d / 2
    let m = n - shift
    let h_len = min(a.len - d, m)
    var h = List.filled(h_len, Mod<P>(0))
    unchecked:
        for i in 0..h_len: h[i] = a[d + i]
    let inv2 = Mod<P>(2).inv()
    var g: List<Mod<P>> = [Mod<P>(r0)]
    var cur = 1
    while cur < m:
        let lim = cur * 2
        var g2 = List.filled(lim, Mod<P>(0))
        unchecked:
            for i in 0..cur: g2[i] = g[i]
        ntt(mut g2, bit_reversed: true)
        unchecked:
            for i in 0..lim: g2[i] = g2[i] * g2[i]
        ntt(mut g2, inverse: true, bit_reversed: true)
        var diff = List.filled(cur, Mod<P>(0))
        unchecked:
            for i in 0..cur:
                let hi = if cur + i < h_len: h[cur + i] else: Mod<P>(0)
                diff[i] = (hi - g2[cur + i]) * inv2
        let ig = inv(g, cur)
        let delta = times(diff, ig)
        unchecked:
            for i in 0..cur: g.push(if i < delta.len: delta[i] else: Mod<P>(0))
        cur = lim
    var out = List.filled(n, Mod<P>(0))
    unchecked:
        for i in 0..m: out[shift + i] = g[i]
    Some(out)

/// `self(x)` by Horner's rule, over any ring (`Mod<P>`, `Int`, `F64`, `Mat`): `coef.eval(x)`
/// with `coef[k]` the coefficient of `x^k`.
///
/// cost: O(self.len) ring operations
/// empty: `T.zero`
/// traps: never (beyond the ring's own arithmetic)
/// example: `[1, 2, 3].eval(2) == 17`
/// example: `[5].eval(100) == 5`
pub fn eval<S: Indexed, T>(self: S, x: T) -> T:
    var acc = T.zero
    for i in (0..self.len).rev: acc = acc * x + self[i]
    acc

fn trim_poly<P>(a: List<Mod<P>>) -> List<Mod<P>>:
    var n = a.len
    unchecked:
        while n > 0 and a[n - 1].v == 0: n -= 1
    var out = List.filled(n, Mod<P>(0))
    unchecked:
        for i in 0..n: out[i] = a[i]
    out

fn rev_poly<P>(a: List<Mod<P>>) -> List<Mod<P>>:
    let n = a.len
    var out = List.filled(n, Mod<P>(0))
    unchecked:
        for i in 0..n: out[i] = a[n - 1 - i]
    out

fn middle_product<P>(a: List<Mod<P>>, b: List<Mod<P>>) -> List<Mod<P>>:
    let na = a.len
    let nb = b.len
    if nb == 0 or na < nb: return []
    let out_len = na - nb + 1
    if nb <= 32 or out_len <= 32:
        var out = List.filled(out_len, Mod<P>(0))
        unchecked:
            for i in 0..out_len:
                var acc = Mod<P>(0)
                for j in 0..nb: acc += a[i + j] * b[j]
                out[i] = acc
        return out
    let size = bit_ceil(na)
    var fa = List.filled(size, Mod<P>(0))
    var fb = List.filled(size, Mod<P>(0))
    unchecked:
        for i in 0..na: fa[i] = a[i]
        for j in 0..nb: fb[j] = b[nb - 1 - j]
    ntt(mut fa, bit_reversed: true)
    ntt(mut fb, bit_reversed: true)
    unchecked:
        for i in 0..size: fa[i] = fa[i] * fb[i]
    ntt(mut fa, inverse: true, bit_reversed: true)
    var out = List.filled(out_len, Mod<P>(0))
    unchecked:
        for i in 0..out_len: out[i] = fa[nb - 1 + i]
    out

/// Euclidean polynomial division `(q, r)` with `a = q * b + r` and `r.len < b.len`.
///
/// `b`'s trailing zeros are ignored, so its last kept coefficient must be invertible. `q` has
/// `max(0, deg a - deg b + 1)` coefficients and `r` has its trailing zeros trimmed (the zero
/// remainder is `[]`).
///
/// cost: O(a.len log a.len)
/// empty: `([], [])` when `a` is empty or zero
/// order: `q` and `r` lowest degree first
/// traps: when `b` is zero (empty or all zeros)
/// example: `type Z = Mod<998244353>; let a: List<Z> = [1, 2, 1]; let b: List<Z> = [1, 1]; div_mod(a, b) == (q: [Z(1), Z(1)], r: [])`
/// example: `type Z = Mod<998244353>; let a: List<Z> = [2, 0, 1]; let b: List<Z> = [1, 1]; div_mod(a, b) == (q: [Z(-1), Z(1)], r: [Z(3)])`
pub fn div_mod<P>(a: List<Mod<P>>, b: List<Mod<P>>) -> (q: List<Mod<P>>, r: List<Mod<P>>):
    let ta = trim_poly(a)
    let tb = trim_poly(b)
    assert tb.len > 0, "poly.div_mod requires a non-zero divisor"
    if ta.len < tb.len: return ([], ta)
    let q_len = ta.len - tb.len + 1
    var ra = List.filled(q_len, Mod<P>(0))
    unchecked:
        for i in 0..q_len: ra[i] = ta[ta.len - 1 - i]
    let rb = rev_poly(tb)
    var q_rev = times(ra, inv(rb, q_len))
    if q_rev.len > q_len: q_rev.truncate(q_len)
    while q_rev.len < q_len: q_rev.push(Mod<P>(0))
    let q = rev_poly(q_rev)
    let qb = times(q, tb)
    let r_len = tb.len - 1
    var r = List.filled(r_len, Mod<P>(0))
    unchecked:
        for i in 0..r_len:
            let ai = if i < ta.len: ta[i] else: Mod<P>(0)
            let qbi = if i < qb.len: qb[i] else: Mod<P>(0)
            r[i] = ai - qbi
    (trim_poly(q), trim_poly(r))

fn subproduct_tree<P>(xs: List<Mod<P>>) -> List<List<Mod<P>>>:
    let n = xs.len
    if n == 0: return [[Mod<P>(1)]]
    var nodes: List<List<Mod<P>>> = List.filled(2 * n, [])
    unchecked:
        for i in 0..n: nodes[n + i] = [-xs[i], Mod<P>(1)]
        for i in (1..n).rev: nodes[i] = times(nodes[2 * i], nodes[2 * i + 1])
    nodes

/// The monic polynomial `prod_{r in roots} (x - r)`, of length `roots.len + 1`.
///
/// cost: O(n log^2 n)
/// empty: `[1]`
/// order: lowest degree first
/// traps: never
/// example: `type Z = Mod<998244353>; let r: List<Z> = [1, 2]; prod_linear(r) == [Z(2), Z(-3), Z(1)]`
pub fn prod_linear<P>(roots: List<Mod<P>>) -> List<Mod<P>>:
    if roots.is_empty: return [Mod<P>(1)]
    subproduct_tree(roots)[1]

/// The Taylor shift `a(x + c)`, of length `a.len`.
///
/// cost: O(n log n)
/// empty: `[]`
/// order: lowest degree first
/// traps: when `a.len > P` (it needs inverse factorials below `a.len`)
/// example: `type Z = Mod<998244353>; let a: List<Z> = [0, 0, 1]; taylor_shift(a, Z(1)) == [Z(1), Z(2), Z(1)]`
pub fn taylor_shift<P>(a: List<Mod<P>>, c: Mod<P>) -> List<Mod<P>>:
    let n = a.len
    if n <= 1 or c.v == 0: return a
    assert n < P, "poly.taylor_shift requires a.len < P"
    var fact = List.filled(n, Mod<P>(1))
    var ifact = List.filled(n, Mod<P>(1))
    unchecked:
        for i in 1..n: fact[i] = fact[i - 1] * Mod<P>(i)
        ifact[n - 1] = fact[n - 1].inv()
        for i in (1..n).rev: ifact[i - 1] = ifact[i] * Mod<P>(i)
    var u = List.filled(n, Mod<P>(0))
    var v = List.filled(n, Mod<P>(0))
    var cp = Mod<P>(1)
    unchecked:
        for i in 0..n:
            u[n - 1 - i] = a[i] * fact[i]
            v[i] = cp * ifact[i]
            cp = cp * c
    let prod = times(u, v)
    var out = List.filled(n, Mod<P>(0))
    unchecked:
        for k in 0..n: out[k] = prod[n - 1 - k] * ifact[k]
    out

/// Multipoint evaluation `[a(x) for x in xs]` via transposed subproduct tree.
///
/// cost: O((a.len + xs.len) log^2 xs.len)
/// empty: `[]` when `xs` is empty; zeros when `a` is empty
/// order: one value per point, in `xs` order
/// traps: when a transform length exceeds what `P` supports
/// example: `type Z = Mod<998244353>; let a: List<Z> = [1, 2, 3]; let xs: List<Z> = [0, 1, 2]; eval_all(a, xs) == [Z(1), Z(6), Z(17)]`
pub fn eval_all<P>(a: List<Mod<P>>, xs: List<Mod<P>>) -> List<Mod<P>>:
    let n = xs.len
    if n == 0: return []
    if a.is_empty: return List.filled(n, Mod<P>(0))
    if n <= 32 or a.len <= 32:
        var out = List.filled(n, Mod<P>(0))
        unchecked:
            for i in 0..n: out[i] = eval(a, xs[i])
        return out
    let nodes = subproduct_tree(xs)
    let root = nodes[1]
    let reduced = if a.len > n: div_mod(a, root).r else: a
    var r_pad = List.filled(n, Mod<P>(0))
    unchecked:
        for i in 0..reduced.len: r_pad[i] = reduced[i]
    let rev_r = rev_poly(r_pad)
    let inv_rev_m = inv(rev_poly(root), n)
    var top = times(rev_r, inv_rev_m)
    if top.len > n: top.truncate(n)
    while top.len < n: top.push(Mod<P>(0))
    var down: List<List<Mod<P>>> = List.filled(2 * n, [])
    down[1] = top
    for i in 1..n:
        down[2 * i] = middle_product(down[i], nodes[2 * i + 1])
        down[2 * i + 1] = middle_product(down[i], nodes[2 * i])
    var out = List.filled(n, Mod<P>(0))
    unchecked:
        for i in 0..n: out[i] = down[n + i][0]
    out

fn biv_advance<P>(mut c: List<Mod<P>>, l_big: Int, l_step: Int) -> List<Mod<P>>:
    let b_sz = 4 << l_big
    let half_b = 2 << l_big
    ntt(mut c, bit_reversed: true)
    var tn = List.filled(b_sz, Mod<P>(0))
    unchecked:
        for i in 0..half_b:
            let i0 = 2 * i
            let i1 = i0 + 1
            let u0 = c[i0]
            let u1 = c[i1]
            tn[i0] = u1
            tn[i1] = u0
            let r = u0 * u1
            c[i0] = r
            c[i1] = r
    ntt(mut c, inverse: true, bit_reversed: true)
    unchecked:
        for j in 1..half_b: c[j] = c[2 * j]
        let span = 2 << (l_big - l_step)
        let stride = 1 << (l_big - l_step)
        for j in 0..span:
            c[half_b + j] = c[j]
            c[j] = Mod<P>(0)
        c[half_b] = c[half_b] - Mod<P>(1)
        c[0] = Mod<P>(1)
        for j in half_b + stride..b_sz: c[j] = Mod<P>(0)
        var off = 0
        while off < half_b:
            for j in 0..stride: c[off + stride + j] = Mod<P>(0)
            off += span
    tn

/// The formal power series composition `f(g(x)) mod x^n` via Kinoshita-Li, of length `n`.
/// Handles any `g[0]` (shifts via `taylor_shift` when `g[0] != 0`). O(n log^2 n).
///
/// cost: O(n log^2 n)
/// empty: `[]` when `n == 0`; zeros when `f` is empty
/// order: lowest degree first
/// traps: when `n < 0`; when a transform length exceeds what `P` supports
/// example: `type Z = Mod<998244353>; let f: List<Z> = [0, 0, 1]; let g: List<Z> = [1, 1]; compose(f, g, 3) == [Z(1), Z(2), Z(1)]`
pub fn compose<P>(f: List<Mod<P>>, g: List<Mod<P>>, n: Int) -> List<Mod<P>>:
    assert n >= 0, "poly.compose requires n >= 0"
    if n == 0: return []
    if f.is_empty: return List.filled(n, Mod<P>(0))
    if g.len > 0 and g[0].v != 0:
        let f_shifted = taylor_shift(f, g[0])
        let g_lim = min(g.len, n)
        var g0 = List.filled(g_lim, Mod<P>(0))
        unchecked:
            for i in 1..g_lim: g0[i] = g[i]
        return compose(f_shifted, g0, n)
    if n == 1: return [f[0]]
    let m = min(f.len, n)
    let k = bit_ceil(n)
    let l_big = ctz(k)
    let b_sz = 4 * k
    let half_b = 2 * k
    var q_arr = List.filled(b_sz, Mod<P>(0))
    q_arr[0] = Mod<P>(1)
    let g_lim = min(g.len, n)
    unchecked:
        for i in 0..g_lim: q_arr[half_b + i] = -g[i]
    let tneg: List<List<Mod<P>>> = [biv_advance(mut q_arr, l_big, l_step) for l_step in 1..=l_big]
    var rev_f = List.filled(m, Mod<P>(0))
    unchecked:
        for i in 0..m: rev_f[i] = f[m - 1 - i]
    var ql = List.filled(m, Mod<P>(0))
    let ql_lim = min(k + 1, m)
    unchecked:
        for i in 0..ql_lim: ql[i] = q_arr[2 * i]
    let p_base = times(rev_f, inv(ql, m))
    var p_arr = List.filled(b_sz, Mod<P>(0))
    unchecked:
        for i in 0..m: p_arr[k - m + i] = p_base[i]
        var idx = k - 1
        while idx > 0:
            p_arr[2 * idx] = p_arr[idx]
            p_arr[idx] = Mod<P>(0)
            idx -= 1
    var l_cur = l_big - 1
    while l_cur >= 0:
        let bit_mask = 1 << (l_big - l_cur)
        unchecked:
            var j = half_b - 1
            while j > 0:
                let v = p_arr[j]
                p_arr[2 * j] = if ((2 * j) & bit_mask) != 0: Mod<P>(0) else: v
                p_arr[j] = Mod<P>(0)
                j -= 1
        ntt(mut p_arr, bit_reversed: true)
        let tn = tneg[l_cur]
        unchecked:
            for j in 0..b_sz: p_arr[j] = tn[j] * p_arr[j]
        ntt(mut p_arr, inverse: true, bit_reversed: true)
        unchecked:
            for j in 0..half_b:
                p_arr[j] = p_arr[half_b + j]
                p_arr[half_b + j] = Mod<P>(0)
        l_cur -= 1
    var out = List.filled(n, Mod<P>(0))
    unchecked:
        for i in 0..n: out[i] = p_arr[i]
    out

/// The value at `x` of the unique polynomial of degree `< ys.len` passing through `(i, ys[i])`
/// for `i in 0..ys.len`. Works for any prime `P` using one modular inverse.
///
/// cost: O(ys.len)
/// empty: `Z(0)` for no points
/// traps: when `ys.len > P` (the sample points `0..ys.len` collide mod `P`)
/// example: `type Z = Mod<998244353>; let ys: List<Z> = [1, 4, 9]; lagrange_at(ys, Z(3)) == Z(16)`
/// example: `type Z = Mod<998244353>; let ys: List<Z> = [1, 4, 9]; lagrange_at(ys, Z(1)) == Z(4)`
pub fn lagrange_at<P>(ys: List<Mod<P>>, x: Mod<P>) -> Mod<P>:
    let n = ys.len
    if n == 0: return Mod<P>(0)
    if x.v < n: return ys[x.v]
    assert n < P, "poly.lagrange_at requires ys.len < P"
    let d = n - 1
    var pref = List.filled(n, Mod<P>(1))
    var ifact = List.filled(n, Mod<P>(1))
    var fact = Mod<P>(1)
    unchecked:
        for i in 0..d: pref[i + 1] = pref[i] * (x - Mod<P>(i))
        for i in 1..n: fact = fact * Mod<P>(i)
        ifact[d] = fact.inv()
        for i in (1..n).rev: ifact[i - 1] = ifact[i] * Mod<P>(i)
        var suff = Mod<P>(1)
        var total = Mod<P>(0)
        for i in (0..n).rev:
            let term = ys[i] * pref[i] * suff * ifact[i] * ifact[d - i]
            total = if ((d - i) & 1) == 0: total + term else: total - term
            suff = suff * (x - Mod<P>(i))
        return total

/// The unique polynomial `f` of degree `< xs.len` with `f(xs[i]) = ys[i]` for distinct `xs`, as
/// exactly `xs.len` coefficients (trailing zeros kept).
/// O(n^2) for `n <= 64` (any prime `P`), O(n log^2 n) via subproduct tree + NTT for `n > 64`.
///
/// cost: O(n^2) for `n <= 64`, O(n log^2 n) above
/// empty: `[]`
/// order: lowest degree first
/// traps: when `xs.len != ys.len`, or two `xs` are equal
/// example: `type Z = Mod<998244353>; let xs: List<Z> = [0, 1, 2]; let ys: List<Z> = [1, 4, 9]; interpolate(xs, ys) == [Z(1), Z(2), Z(1)]`
/// example: `type Z = Mod<998244353>; let xs: List<Z> = [0, 1]; let ys: List<Z> = [5, 5]; interpolate(xs, ys) == [Z(5), Z(0)]`
pub fn interpolate<P>(xs: List<Mod<P>>, ys: List<Mod<P>>) -> List<Mod<P>>:
    let n = xs.len
    assert n == ys.len, "poly.interpolate requires equal-length xs and ys"
    if n == 0: return []
    if n == 1: return [ys[0]]
    if n <= 64:
        var prod = List.filled(n + 1, Mod<P>(0))
        prod[0] = Mod<P>(1)
        unchecked:
            for i in 0..n:
                let xi = xs[i]
                for k in (1..=i + 1).rev: prod[k] = prod[k - 1] - prod[k] * xi
                prod[0] = -prod[0] * xi
            var out = List.filled(n, Mod<P>(0))
            for i in 0..n:
                let xi = xs[i]
                var denom = Mod<P>(1)
                for j in 0..n:
                    if j != i: denom = denom * (xi - xs[j])
                assert denom.v != 0, "poly.interpolate requires distinct xs"
                let scale = ys[i] * denom.inv()
                var rem = Mod<P>(0)
                for k in (0..n).rev:
                    rem = prod[k + 1] + rem * xi
                    out[k] = out[k] + rem * scale
            return out
    let nodes = subproduct_tree(xs)
    let w = eval_all(deriv(nodes[1]), xs)
    var num: List<List<Mod<P>>> = List.filled(2 * n, [])
    unchecked:
        for i in 0..n:
            assert w[i].v != 0, "poly.interpolate requires distinct xs"
            num[n + i] = [ys[i] * w[i].inv()]
        for i in (1..n).rev:
            let l = times(num[2 * i], nodes[2 * i + 1])
            let r = times(num[2 * i + 1], nodes[2 * i])
            let m = max(l.len, r.len)
            var res = List.filled(m, Mod<P>(0))
            for k in 0..l.len: res[k] = res[k] + l[k]
            for k in 0..r.len: res[k] = res[k] + r[k]
            num[i] = res
    var out = List.filled(n, Mod<P>(0))
    let root = num[1]
    let lim = min(n, root.len)
    unchecked:
        for i in 0..lim: out[i] = root[i]
    out

/// O(n) storage. Algebraic polynomial / formal power series wrapper over `T` (lowest degree first).
pub struct Poly<T>(c: List<T>):
    /// A polynomial with coefficients `c`, lowest degree first, stored as given (trailing zeros
    /// kept, so `len` is `c.len`).
    ///
    /// cost: O(n)
    /// empty: `Poly.new([])` is the zero polynomial
    /// traps: never
    /// example: `Poly<Int>([1, 2, 0]).len == 3`
    pub fn new(c: List<T>) -> Poly<T> = todo

    /// The constant polynomial `v`, with one coefficient `T.from_int(v)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Poly<Int>.from_int(5).at(0) == 5`
    pub fn from_int(v: Int) -> Poly<T> = todo

    /// The zero polynomial `0`, with no coefficients.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Poly<Int>.zero.deg() == -1 and Poly<Int>.zero.len == 0`
    @get pub fn zero() -> Poly<T> = todo

    /// The constant polynomial `1`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Poly<Int>.one.at(0) == 1 and Poly<Int>.one.deg() == 0`
    @get pub fn one() -> Poly<T> = todo

    /// The indeterminate `x`: coefficients `[0, 1]`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Poly<Int>.x.deg() == 1 and Poly<Int>.x.at(1) == 1`
    @get pub fn x() -> Poly<T> = todo

    /// The number of stored coefficients, trailing zeros included.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Poly<Int>([1, 0]).len == 2`
    @get pub fn len(self) -> Int = todo

    /// The degree: the highest index with a nonzero coefficient, `-1` for the zero polynomial. Stored
    /// trailing zeros do not count.
    ///
    /// cost: O(1) plus the number of stored trailing zeros
    /// traps: never
    /// example: `Poly<Int>([1, 2, 0]).deg() == 1`
    /// example: `Poly<Int>([0, 0]).deg() == -1`
    pub fn deg(self) -> Int = todo

    /// Whether all coefficients are zero (including when there are None).
    ///
    /// cost: O(1) plus the number of stored trailing zeros
    /// traps: never
    /// example: `Poly<Int>([0, 0]).is_zero() and not Poly<Int>.one.is_zero()`
    pub fn is_zero(self) -> Bool = todo

    /// Read coefficient `i` (0 when `i >= self.len`).
    ///
    /// cost: O(1)
    /// traps: when `i < 0`
    /// example: `Poly<Int>([1, 2]).at(1) == 2 and Poly<Int>([1, 2]).at(5) == 0`
    pub fn at(self, i: Int) -> T = todo

    /// Write coefficient `i`, growing with zeros if needed.
    ///
    /// cost: O(1), or O(i) when it grows
    /// traps: when `i < 0`
    /// example: `var p = Poly<Int>([1]); p.set(3, 7); p.len == 4 and p.at(2) == 0 and p.at(3) == 7`
    pub fn set(mut self, i: Int, v: T) = todo

    /// The formal derivative `f'(x)`, as the free `deriv` does on the coefficients.
    ///
    /// cost: O(n)
    /// traps: never
    /// example: `Poly<Int>([1, 2, 3]).deriv().at(1) == 6`
    pub fn deriv(self) -> Poly<T> = todo

    /// The formal integral with zero constant term, as the free `integral`.
    ///
    /// cost: O(n)
    /// traps: when `T` cannot divide by some `k` in `1..=n` (`n >= P` for `Mod<P>`)
    /// example: `type Z = Mod<998244353>; Poly<Z>([1, 2, 3]).integral().at(3) == Z(1)`
    pub fn integral(self) -> Poly<T> = todo

    /// The transposed (middle) product: `out[i] = sum_j self[i + j] * b[j]` over `j in 0..b.len`, for
    /// each `i` in `0..max(0, self.len - b.len + 1)` (only complete sums).
    ///
    /// cost: O((n + m) log(n + m))
    /// traps: never
    /// example: `Poly<Int>([1, 2, 3, 4]).transposed_mul(Poly<Int>([1, 1])).len == 3`
    /// example: `Poly<Int>([1, 2, 3, 4]).transposed_mul(Poly<Int>([1, 1])).at(2) == 7`
    pub fn transposed_mul(self, b: Poly<T>) -> Poly<T> = todo

    /// Truncate or zero-pad to exactly `n` coefficients (mod `x^n`).
    ///
    /// cost: O(n)
    /// traps: when `n < 0`
    /// example: `Poly<Int>([1, 2, 3]).truncated(2).len == 2`
    /// example: `Poly<Int>([1]).truncated(3).at(2) == 0`
    pub fn truncated(self, n: Int) -> Poly<T> = todo

    /// The multiplicative inverse mod `x^n` (`self[0] != 0`), as the free `inv`.
    ///
    /// cost: O(n log n)
    /// traps: as the free `inv`: `n < 0`, or `n > 0` with `self.at(0) == 0`
    /// example: `type Z = Mod<998244353>; Poly<Z>([1, -1]).inv(3).at(2) == Z(1)`
    pub fn inv(self, n: Int) -> Poly<T> = todo

    /// The formal logarithm mod `x^n` (`self[0] == 1`), as the free `log`.
    ///
    /// cost: O(n log n)
    /// traps: as the free `log`: `n < 0`, or `n > 0` with `self.at(0) != 1`
    /// example: `type Z = Mod<998244353>; Poly<Z>([1, 2, 1]).log(3).at(2) == Z(-1)`
    pub fn log(self, n: Int) -> Poly<T> = todo

    /// The formal exponential mod `x^n` (`self[0] == 0`), as the free `exp`.
    ///
    /// cost: O(n log n)
    /// traps: as the free `exp`: `n < 0`, or `n > 0` with `self.at(0) != 0`
    /// example: `type Z = Mod<998244353>; Poly<Z>([0, 2]).exp(3).at(2) == Z(2)`
    pub fn exp(self, n: Int) -> Poly<T> = todo

    /// The formal power `self(x)^k mod x^n`, as the free `pow`.
    ///
    /// cost: O(n log n)
    /// traps: as the free `pow`: `n < 0`, or `k < 0` with `self.at(0) == 0`
    /// example: `type Z = Mod<998244353>; Poly<Z>([1, 1]).pow(3, 4).at(2) == Z(3)`
    pub fn pow(self, k: Int, n: Int) -> Poly<T> = todo

    /// The formal square root mod `x^n`, or `None`, as the free `sqrt`.
    ///
    /// cost: O(n log n + log^2 P)
    /// traps: when `n < 0`
    /// example: `type Z = Mod<998244353>; Poly<Z>([1, 2, 1]).sqrt(3)!.at(1) == Z(1)`
    pub fn sqrt(self, n: Int) -> ?Poly<T> = todo

    /// The value at `x` by Horner's rule.
    ///
    /// cost: O(n)
    /// traps: never
    /// example: `Poly<Int>([1, 2, 3]).eval(2) == 17`
    pub fn eval(self, x: T) -> T = todo

    /// The Taylor shift `self(x + c)`, as `taylor_shift`.
    ///
    /// cost: O(n log n)
    /// traps: when `T` cannot divide by the factorials below `n` (`n > P` for `Mod<P>`)
    /// example: `type Z = Mod<998244353>; Poly<Z>([0, 0, 1]).shift(Z(1)).at(1) == Z(2)`
    pub fn shift(self, c: T) -> Poly<T> = todo

impl Add for Poly<T>:
    fn add(a, b) = todo

impl Sub for Poly<T>:
    fn sub(a, b) = todo

impl Neg for Poly<T>:
    fn neg(a) = todo

impl Mul for Poly<T>:
    fn mul(a, b) = todo

impl Div for Poly<T>:
    fn div(a, b) = todo

/// O(m log^2 m) build. Prebuilt product tree over points `xs` for multipoint evaluation,
/// transposed evaluation, and interpolation.
pub struct SubproductTree<P>:
    n: Int

    /// Build the subproduct tree for the points `xs`: leaf `i` is `x - xs[i]` and each node the
    /// product of its children. With `keep >= 0` every stored product is truncated mod `x^keep`, which
    /// is all `eval_transposed` needs; `eval` and `interpolate` need the full tree (`keep < 0`, the
    /// default).
    ///
    /// cost: O(m log^2 m) time, O(m log m) space
    /// empty: a tree over no points
    /// traps: when a transform length exceeds what `P` supports
    /// example: `type Z = Mod<998244353>; let xs: List<Z> = [0, 1, 2]; let t = SubproductTree(xs); t.eval([Z(1), Z(2), Z(3)]) == [Z(1), Z(6), Z(17)]`
    pub fn new(xs: List<Mod<P>>, keep: Int = -1) -> SubproductTree<P> = todo

    /// Evaluate `f` at every point in the tree: `[f(x) for x in xs]`.
    ///
    /// cost: O((deg f + m) log^2 m)
    /// empty: `[]` for a tree over no points
    /// order: one value per point, in `xs` order
    /// traps: when the tree was built with `keep >= 0`
    /// example: `type Z = Mod<998244353>; let xs: List<Z> = [1, 2]; SubproductTree(xs).eval([Z(0), Z(0), Z(1)]) == [Z(1), Z(4)]`
    pub fn eval(self, f: List<Mod<P>>) -> List<Mod<P>> = todo

    /// Transposed multipoint evaluation (Bostan-Lecerf-Schost / Tellegen's principle): the transpose
    /// of `eval` on polynomials of degree `< m`, so `out[k] = sum_i c[i] * xs[i]^k` for `k in 0..m`.
    ///
    /// cost: O(m log^2 m)
    /// empty: `[]` for a tree over no points
    /// order: by power `k`, increasing
    /// traps: when `c.len != m`, or `keep` is in `0..m`
    /// example: `type Z = Mod<998244353>; let xs: List<Z> = [1, 2]; SubproductTree(xs).eval_transposed([Z(1), Z(1)]) == [Z(2), Z(3)]`
    pub fn eval_transposed(self, c: List<Mod<P>>) -> List<Mod<P>> = todo

    /// The unique polynomial of degree `< m` taking values `ys` at the tree's points, as exactly `m`
    /// coefficients. The points must be distinct.
    ///
    /// cost: O(m log^2 m)
    /// empty: `[]`
    /// order: lowest degree first
    /// traps: when `ys.len != m`, two points are equal, or the tree was built with `keep >= 0`
    /// example: `type Z = Mod<998244353>; let xs: List<Z> = [0, 1, 2]; SubproductTree(xs).interpolate([Z(1), Z(4), Z(9)]) == [Z(1), Z(2), Z(1)]`
    pub fn interpolate(self, ys: List<Mod<P>>) -> List<Mod<P>> = todo

/// The exact coefficient `[x^n] (P(x) / Q(x))` via Bostan-Mori (`q[0] != 0`): the `n`-th term of
/// the linear recurrence with denominator `q` and numerator `p`.
///
/// cost: O(k log k log n) for `k = max(p.len, q.len)`
/// empty: zero when `p` is empty
/// traps: when `n < 0`, or `q` is empty or `q[0] == 0`
/// example: `type Z = Mod<998244353>; let p: List<Z> = [1]; let q: List<Z> = [1, -1, -1]; bostan_mori(p, q, 5) == Z(8)`
pub fn bostan_mori<P>(p: List<Mod<P>>, q: List<Mod<P>>, n: Int) -> Mod<P> = todo

/// The transposed (middle) product: `out[i] = sum_j a[i + j] * b[j]` over `j in 0..b.len`, for each
/// `i` in `0..max(0, a.len - b.len + 1)` (only complete sums).
///
/// cost: O((n + m) log(n + m))
/// empty: `[]` when `b.len > a.len`
/// order: by `i`, increasing
/// traps: when a transform length exceeds what `P` supports
/// example: `type Z = Mod<998244353>; let a: List<Z> = [1, 2, 3, 4]; let b: List<Z> = [1, 1]; transposed_mul(a, b) == [Z(3), Z(5), Z(7)]`
pub fn transposed_mul<P>(a: List<Mod<P>>, b: List<Mod<P>>) -> List<Mod<P>> = todo

/// The compositional inverse `g` with `f(g(x)) == x (mod x^n)` (`f[0] == 0, f[1] != 0`), of length
/// `n`; `g[0] == 0`.
///
/// cost: O(n log^2 n)
/// empty: `[]` when `n == 0`
/// order: lowest degree first
/// traps: when `n < 0`; when `n > 0` and `f.len < 2`, `f[0] != 0` or `f[1] == 0`
/// example: `type Z = Mod<998244353>; let f: List<Z> = [0, 1, 1]; compose_inv(f, 4) == [Z(0), Z(1), Z(-1), Z(2)]`
pub fn compose_inv<P>(f: List<Mod<P>>, n: Int) -> List<Mod<P>> = todo

/// The polynomial power `a(x)^k mod mod_p(x)` where `d = mod_p.len`, by binary powering with
/// `div_mod`. The result has exactly `deg mod_p` coefficients; `mod_p`'s trailing zeros are
/// ignored.
///
/// cost: O(d log d log k)
/// empty: `[]` when `mod_p` is a nonzero constant
/// order: lowest degree first
/// traps: when `k < 0`, or `mod_p` is zero
/// example: `type Z = Mod<998244353>; let a: List<Z> = [0, 1]; let m: List<Z> = [-1, 0, 1]; pow_mod_poly(a, 3, m) == [Z(0), Z(1)]`
/// example: `type Z = Mod<998244353>; let a: List<Z> = [0, 1]; let m: List<Z> = [-1, 0, 1]; pow_mod_poly(a, 0, m) == [Z(1), Z(0)]`
pub fn pow_mod_poly<P>(a: List<Mod<P>>, k: Int, mod_p: List<Mod<P>>) -> List<Mod<P>> = todo

/// The monic polynomial GCD via half-GCD, with trailing zeros trimmed; the gcd of two zero
/// polynomials is the zero polynomial `[]`.
///
/// cost: O(n log^2 n)
/// empty: `gcd_poly([], b)` is `b` made monic
/// order: lowest degree first
/// traps: never
/// example: `type Z = Mod<998244353>; let a: List<Z> = [2, -3, 1]; let b: List<Z> = [3, -4, 1]; gcd_poly(a, b) == [Z(-1), Z(1)]`
/// example: `type Z = Mod<998244353>; let a: List<Z> = [1, 1]; let b: List<Z> = [2]; gcd_poly(a, b) == [Z(1)]`
pub fn gcd_poly<P>(a: List<Mod<P>>, b: List<Mod<P>>) -> List<Mod<P>> = todo

/// Bluestein / Chirp-Z evaluation `[a(r^0), a(r^1), ..., a(r^(m - 1))]`.
///
/// cost: O((n + m) log(n + m))
/// empty: `[]` when `m == 0`; zeros when `a` is empty
/// order: by power of `r`, increasing
/// traps: when `m < 0`
/// example: `type Z = Mod<998244353>; let a: List<Z> = [1, 1]; chirp_z(a, Z(2), 3) == [Z(2), Z(3), Z(5)]`
pub fn chirp_z<P>(a: List<Mod<P>>, r: Mod<P>, m: Int) -> List<Mod<P>> = todo

/// Given `f(0..n)` for `deg f < n`, return `[f(c), f(c + 1), ..., f(c + m - 1)]`, where `n` is
/// `ys.len`.
///
/// cost: O((n + m) log(n + m))
/// empty: `[]` when `m == 0`; zeros when `ys` is empty
/// order: by point, increasing
/// traps: when `m < 0`, or `n + m > P` (the sample points collide mod `P`)
/// example: `type Z = Mod<998244353>; let ys: List<Z> = [0, 1, 4]; sample_points_shift(ys, Z(3), 2) == [Z(9), Z(16)]`
pub fn sample_points_shift<P>(ys: List<Mod<P>>, c: Mod<P>, m: Int) -> List<Mod<P>> = todo

/// The power sums `sum_i xs[i]^k` for `k in 0..=deg` via `-x p'(x) / p(x)`; entry 0 is `xs.len`.
///
/// cost: O((n + deg) log^2 n)
/// empty: zeros when `xs` is empty
/// order: by `k`, increasing
/// traps: when `deg < 0`
/// example: `type Z = Mod<998244353>; let xs: List<Z> = [1, 2]; power_sums(xs, 2) == [Z(2), Z(3), Z(5)]`
pub fn power_sums<P>(xs: List<Mod<P>>, deg: Int) -> List<Mod<P>> = todo

/// The unsigned Stirling numbers of the first kind `s(n, 0..=n)`: the coefficients of
/// `x (x + 1) ... (x + n - 1)`.
///
/// cost: O(n log n)
/// empty: `[1]` for `n == 0`
/// order: by `k`, increasing
/// traps: when `n < 0`
/// example: `type Z = Mod<998244353>; let s: List<Z> = stirling1_row(3); s == [Z(0), Z(2), Z(3), Z(1)]`
pub fn stirling1_row<P>(n: Int) -> List<Mod<P>> = todo

/// The Stirling numbers of the second kind `S(n, 0..=n)`.
///
/// cost: O(n log n)
/// empty: `[1]` for `n == 0`
/// order: by `k`, increasing
/// traps: when `n < 0`, or `n >= P` (it divides by factorials up to `n`)
/// example: `type Z = Mod<998244353>; let s: List<Z> = stirling2_row(3); s == [Z(0), Z(1), Z(3), Z(1)]`
pub fn stirling2_row<P>(n: Int) -> List<Mod<P>> = todo

/// The unsigned Stirling numbers of the first kind column `s(0..=n, k)`, `n + 1` values (zero for
/// rows below `k`).
///
/// cost: O(n log n)
/// empty: zeros when `k > n`
/// order: by row, increasing
/// traps: when `n < 0`, `k < 0`, or `n >= P`
/// example: `type Z = Mod<998244353>; let s: List<Z> = stirling1_col(1, 3); s == [Z(0), Z(1), Z(1), Z(2)]`
pub fn stirling1_col<P>(k: Int, n: Int) -> List<Mod<P>> = todo

/// The Stirling numbers of the second kind column `S(0..=n, k)`, `n + 1` values (zero for rows
/// below `k`).
///
/// cost: O(n log n)
/// empty: zeros when `k > n`
/// order: by row, increasing
/// traps: when `n < 0`, `k < 0`, or `n >= P`
/// example: `type Z = Mod<998244353>; let s: List<Z> = stirling2_col(2, 4); s == [Z(0), Z(0), Z(1), Z(3), Z(7)]`
pub fn stirling2_col<P>(k: Int, n: Int) -> List<Mod<P>> = todo

/// The Bell numbers `B_0..=B_n`, `n + 1` values.
///
/// cost: O(n log n)
/// empty: `[1]` for `n == 0`
/// order: by index, increasing
/// traps: when `n < 0`, or `n >= P`
/// example: `type Z = Mod<998244353>; let b: List<Z> = bell_table(4); b == [Z(1), Z(1), Z(2), Z(5), Z(15)]`
pub fn bell_table<P>(n: Int) -> List<Mod<P>> = todo

/// The Bernoulli numbers `B_0..=B_n` (`B_1 = -1/2`), `n + 1` values.
///
/// cost: O(n log n)
/// empty: `[1]` for `n == 0`
/// order: by index, increasing
/// traps: when `n < 0`, or `n + 1 >= P`
/// example: `type Z = Mod<998244353>; let b: List<Z> = bernoulli_table(2); b == [Z(1), Z(-1) / Z(2), Z(1) / Z(6)]`
pub fn bernoulli_table<P>(n: Int) -> List<Mod<P>> = todo

/// The partition numbers `p(0..=n)` via Euler's pentagonal theorem, `n + 1` values.
///
/// cost: O(n log n)
/// empty: `[1]` for `n == 0`
/// order: by index, increasing
/// traps: when `n < 0`
/// example: `type Z = Mod<998244353>; let p: List<Z> = partition_table(5); p == [Z(1), Z(1), Z(2), Z(3), Z(5), Z(7)]`
pub fn partition_table<P>(n: Int) -> List<Mod<P>> = todo

@test
fn eval_div_shift_mpe_laws():
    type Z = Mod<998_244_353>
    let a = [Z((i * 11 + 3) % 29) for i in 0..45]
    let b = [Z((i * 7 + 5) % 23) for i in 0..19]
    let (q, r) = div_mod(a, b)
    assert r.len < b.len
    let qb = times(q, b)
    for i in 0..a.len:
        let qi = if i < qb.len: qb[i] else: Z(0)
        let ri = if i < r.len: r[i] else: Z(0)
        assert qi + ri == a[i]

    let roots = [Z(i * 3 + 1) for i in 0..25]
    let pl = prod_linear(roots)
    assert pl.len == roots.len + 1
    assert pl[roots.len] == Z(1)
    for x in roots: assert eval(pl, x) == Z(0)
    assert eval(pl, Z(100)) == prod(Z(100) - x for x in roots)

    let c = Z(17)
    let sh = taylor_shift(a, c)
    for k in 0..10:
        let x = Z(k * 13 + 2)
        assert eval(sh, x) == eval(a, x + c)

    let xs = [Z(i * 19 + 4) for i in 0..40]
    let vals = eval_all(a, xs)
    for i in 0..xs.len: assert vals[i] == eval(a, xs[i])
    let vals_short = eval_all(b, xs)
    for i in 0..xs.len: assert vals_short[i] == eval(b, xs[i])

    // Negative power law: a^k * a^{-k} == 1 (mod x^n)
    let p_pos = pow(b, 5, 16)
    let p_neg = pow(b, -5, 16)
    let p_id = times(p_pos, p_neg)
    assert p_id[0] == Z(1)
    for i in 1..16: assert p_id[i] == Z(0)

    // Composition law against Horner brute force (both g[0] == 0 and g[0] != 0)
    let f_c: List<Z> = [Z(3), Z(5), Z(7), Z(11), Z(13), Z(17)]
    for g0 in [Z(0), Z(4)]:
        let g_c: List<Z> = [g0, Z(2), Z(9), Z(1), Z(6)]
        let comp = compose(f_c, g_c, 12)
        var brute = List.filled(12, Z(0))
        var gp: List<Z> = [Z(1)]
        for i in 0..f_c.len:
            for j in 0..min(gp.len, 12): brute[j] = brute[j] + f_c[i] * gp[j]
            gp = times(gp, g_c)
            if gp.len > 12: gp.truncate(12)
        assert comp == brute

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
    let prod = times(a, ia)
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
        naive = times(naive, a)
        if naive.len > 10: naive.truncate(10)
    while naive.len < 10: naive.push(Z(0))
    assert p3 == naive
    assert pow(a, 0, 4) == [Z(1), Z(0), Z(0), Z(0)]
    assert pow(a, 1_000_000_000_000, 6) == List.filled(6, Z(0))

    let s_in: List<Z> = [Z(0), Z(0), Z(4), Z(12), Z(9), Z(5), Z(7)]
    let g = sqrt(s_in, 7)!
    let gg = times(g, g)
    for i in 0..7: assert gg[i] == s_in[i]
    let odd_zero: List<Z> = [Z(0), Z(4), Z(1)]
    assert sqrt(odd_zero, 3) == None
    let non_res: List<Z> = [Z(3), Z(1)]
    assert sqrt(non_res, 2) == None
    assert sqrt(List.filled(5, Z(0)), 5)! == List.filled(5, Z(0))

@test
fn lagrange_and_interpolate_laws():
    type M = Mod<1_000_000_007>
    var faulhaber = [M(0)]
    var acc = M(0)
    for i in 1..=6:
        acc += M(i) ** 4
        faulhaber.push(acc)
    var brute = M(0)
    for i in 1..=50: brute += M(i) ** 4
    assert lagrange_at(faulhaber, M(50)) == brute
    for i in 0..faulhaber.len: assert lagrange_at(faulhaber, M(i)) == faulhaber[i]
    type Z = Mod<998_244_353>
    let a = [Z((i * 11 + 3) % 29) for i in 0..20]
    let ys_a = [eval(a, Z(i)) for i in 0..a.len]
    for x in [Z(0), Z(7), Z(20), Z(123_456)]: assert lagrange_at(ys_a, x) == eval(a, x)
    let xs_s = [Z(i * 3 + 1) for i in 0..12]
    let ys_s = [eval(a[0..12], x) for x in xs_s]
    assert interpolate(xs_s, ys_s) == a[0..12]
    let poly_big = [Z((i * 19 + 7) % 101) for i in 0..70]
    let xs_b = [Z(i * 5 + 2) for i in 0..70]
    let ys_b = eval_all(poly_big, xs_b)
    assert interpolate(xs_b, ys_b) == poly_big
