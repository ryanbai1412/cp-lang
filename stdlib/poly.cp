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

/// Formal power `a(x)^k mod x^n` (supports `k < 0` when `a[0] != 0`). O(n log n).
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
    let ck = c ** k
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
        let delta = convolve(diff, ig)
        unchecked:
            for i in 0..cur: g.push(if i < delta.len: delta[i] else: Mod<P>(0))
        cur = lim
    var out = List.filled(n, Mod<P>(0))
    unchecked:
        for i in 0..m: out[shift + i] = g[i]
    some(out)

/// Evaluates `a(x)` via Horner's method. O(a.len).
pub fn eval<P>(a: List<Mod<P>>, x: Mod<P>) -> Mod<P>:
    var acc = Mod<P>(0)
    unchecked:
        for i in (0..a.len).rev: acc = acc * x + a[i]
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
/// Requires `b` to be non-zero. O(a.len log a.len).
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
    var q_rev = convolve(ra, inv(rb, q_len))
    if q_rev.len > q_len: q_rev.truncate(q_len)
    while q_rev.len < q_len: q_rev.push(Mod<P>(0))
    let q = rev_poly(q_rev)
    let qb = convolve(q, tb)
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
        for i in (1..n).rev: nodes[i] = convolve(nodes[2 * i], nodes[2 * i + 1])
    nodes

/// Monic polynomial `prod_{r in roots} (x - r)`. O(n log^2 n).
pub fn prod_linear<P>(roots: List<Mod<P>>) -> List<Mod<P>>:
    if roots.is_empty: return [Mod<P>(1)]
    subproduct_tree(roots)[1]

/// Taylor shift `a(x + c)`. Length `a.len`. O(n log n).
pub fn taylor_shift<P>(a: List<Mod<P>>, c: Mod<P>) -> List<Mod<P>>:
    let n = a.len
    if n <= 1 or c.v == 0: return a
    assert n < P, "poly.taylor_shift requires a.len < P"
    var fact = List.filled(n, Mod<P>(1))
    var ifact = List.filled(n, Mod<P>(1))
    unchecked:
        for i in 1..n: fact[i] = fact[i - 1] * Mod<P>(i)
        ifact[n - 1] = fact[n - 1].inv
        for i in (1..n).rev: ifact[i - 1] = ifact[i] * Mod<P>(i)
    var u = List.filled(n, Mod<P>(0))
    var v = List.filled(n, Mod<P>(0))
    var cp = Mod<P>(1)
    unchecked:
        for i in 0..n:
            u[n - 1 - i] = a[i] * fact[i]
            v[i] = cp * ifact[i]
            cp = cp * c
    let prod = convolve(u, v)
    var out = List.filled(n, Mod<P>(0))
    unchecked:
        for k in 0..n: out[k] = prod[n - 1 - k] * ifact[k]
    out

/// Multipoint evaluation `[a(x) for x in xs]` via transposed subproduct tree.
/// O((a.len + xs.len) log^2 xs.len).
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
    var top = convolve(rev_r, inv_rev_m)
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

/// Formal power series composition `f(g(x)) mod x^n` via Kinoshita-Li.
/// Handles any `g[0]` (shifts via `taylor_shift` when `g[0] != 0`). O(n log^2 n).
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
    let p_base = convolve(rev_f, inv(ql, m))
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

/// Value at `x` of the unique polynomial of degree `< ys.len` passing through `(i, ys[i])`
/// for `i in 0..ys.len`. Works for any prime `P` using one modular inverse. O(ys.len).
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
        ifact[d] = fact.inv
        for i in (1..n).rev: ifact[i - 1] = ifact[i] * Mod<P>(i)
        var suff = Mod<P>(1)
        var total = Mod<P>(0)
        for i in (0..n).rev:
            let term = ys[i] * pref[i] * suff * ifact[i] * ifact[d - i]
            total = if ((d - i) & 1) == 0: total + term else: total - term
            suff = suff * (x - Mod<P>(i))
        return total

/// Unique polynomial `f` of degree `< xs.len` with `f(xs[i]) = ys[i]` for distinct `xs`.
/// O(n^2) for `n <= 64` (any prime `P`), O(n log^2 n) via subproduct tree + NTT for `n > 64`.
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
                let scale = ys[i] * denom.inv
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
            num[n + i] = [ys[i] * w[i].inv]
        for i in (1..n).rev:
            let l = convolve(num[2 * i], nodes[2 * i + 1])
            let r = convolve(num[2 * i + 1], nodes[2 * i])
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

@test
fn eval_div_shift_mpe_laws():
    type Z = Mod<998_244_353>
    let a = [Z((i * 11 + 3) % 29) for i in 0..45]
    let b = [Z((i * 7 + 5) % 23) for i in 0..19]
    let (q, r) = div_mod(a, b)
    assert r.len < b.len
    let qb = convolve(q, b)
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
    let p_id = convolve(p_pos, p_neg)
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
            gp = convolve(gp, g_c)
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
