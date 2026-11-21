//! Number-theoretic transform, exact and modular convolution, SOS, and bitwise transforms.
//!
//! Polynomials are coefficient lists, lowest degree first (`p[k]` is the coefficient of `x^k`).
//! `convolve`, `convolve_mod`, and `convolve_int` take `cyclic: Bool = false`:
//! - linear (`cyclic: false`): length `a.len + b.len - 1` (or `[]` if either input is empty)
//! - cyclic (`cyclic: true`): requires `a.len == b.len`, length `n = a.len`, index `(i + j) % n`.
use std.math.{is_pow2, mul_mod, pow_mod, primitive_root}
use std.mod.{Mod}

const P1 = 998_244_353
const P2 = 469_762_049
const P3 = 167_772_161

fn root_of(p: Int) -> Int =
    if p == P1 or p == P2 or p == P3: 3
    elif p == 754_974_721: 11
    else: primitive_root(p)

fn mont_inv(p: Int) -> Int:
    var x: U64 = U64(p)
    let pu = U64(p)
    for _ in 0..4: x = x * (U64(2) - pu * x)
    Int((U64(0) - x) & U64(4_294_967_295))

fn mred(x: Int, p: Int, pinv: Int) -> Int:
    let r = (x + (((x & 4_294_967_295) * pinv) & 4_294_967_295) * p) >> 32
    if r >= p: r - p else: r

fn bit_reverse<T>(mut a: List<T>):
    let n = a.len
    unchecked:
        var j = 0
        for i in 1..n:
            var bit = n >> 1
            while (j & bit) != 0:
                j ^= bit
                bit >>= 1
            j ^= bit
            if i < j:
                let tmp = a[i]
                a[i] = a[j]
                a[j] = tmp

fn ntt_raw(mut a: List<Int>, p: Int, g: Int, inverse: Bool):
    let n = a.len
    assert n.is_pow2, "ntt requires a power-of-two length"
    if n <= 1: return
    assert (p - 1) % n == 0, "modulus p - 1 must be divisible by n"
    bit_reverse(mut a)
    if p < 1_073_741_824:
        let pinv = mont_inv(p)
        let r1 = (1 << 32) % p
        let r2 = r1 * r1 % p
        var roots = List.filled(n >> 1, 0)
        unchecked:
            for i in 0..n: a[i] = mred(a[i] * r2, p, pinv)
            var len = 2
            while len <= n:
                let half = len >> 1
                var wlen = pow_mod(g, (p - 1) / len, p)
                if inverse: wlen = pow_mod(wlen, p - 2, p)
                let wlen_m = mred(wlen * r2, p, pinv)
                roots[0] = r1
                for k in 1..half: roots[k] = mred(roots[k - 1] * wlen_m, p, pinv)
                var i = 0
                while i < n:
                    for k in 0..half:
                        let u = a[i + k]
                        let v = mred(a[i + k + half] * roots[k], p, pinv)
                        var x = u + v
                        if x >= p: x -= p
                        var y = u - v
                        if y < 0: y += p
                        a[i + k] = x
                        a[i + k + half] = y
                    i += len
                len <<= 1
            if inverse:
                let inv_n = pow_mod(n, p - 2, p)
                for i in 0..n: a[i] = mred(a[i] * inv_n, p, pinv)
            else:
                for i in 0..n: a[i] = mred(a[i], p, pinv)
        return
    let small = p <= 3_037_000_499
    unchecked:
        var len = 2
        while len <= n:
            let half = len >> 1
            var wlen = pow_mod(g, (p - 1) / len, p)
            if inverse: wlen = pow_mod(wlen, p - 2, p)
            var i = 0
            while i < n:
                var w = 1
                for k in 0..half:
                    let u = a[i + k]
                    let v = if small: a[i + k + half] * w % p else: mul_mod(a[i + k + half], w, p)
                    var x = u + v
                    if x >= p: x -= p
                    var y = u - v
                    if y < 0: y += p
                    a[i + k] = x
                    a[i + k + half] = y
                    w = if small: w * wlen % p else: mul_mod(w, wlen, p)
                i += len
            len <<= 1
        if inverse:
            let inv_n = pow_mod(n, p - 2, p)
            for i in 0..n: a[i] = if small: a[i] * inv_n % p else: mul_mod(a[i], inv_n, p)

fn convolve_prime(a: List<Int>, b: List<Int>, p: Int, cyclic: Bool) -> List<Int>:
    if a.is_empty or b.is_empty: return []
    let na = a.len
    let nb = b.len
    if cyclic:
        assert na == nb, "cyclic convolution requires equal-length inputs"
    let n_out = if cyclic: na else: na + nb - 1
    if min(na, nb) <= 60:
        var out = List.filled(n_out, 0)
        let small = p <= 3_037_000_499
        for i in 0..na:
            let ai = a[i] % p
            if ai == 0: continue
            for j in 0..nb:
                let bj = b[j] % p
                let prod = if small: ai * bj % p else: mul_mod(ai, bj, p)
                let idx = if cyclic: (i + j) % na else: i + j
                var s = out[idx] + prod
                if s >= p: s -= p
                out[idx] = s
        return out
    let g = root_of(p)
    let need = if cyclic and na.is_pow2 and (p - 1) % na == 0: na else: bit_ceil(na + nb - 1)
    assert (p - 1) % need == 0, "modulus p - 1 must be divisible by n"
    var fa = List.filled(need, 0)
    var fb = List.filled(need, 0)
    unchecked:
        for i in 0..na: fa[i] = a[i] % p
        for i in 0..nb: fb[i] = b[i] % p
    ntt_raw(mut fa, p, g, false)
    ntt_raw(mut fb, p, g, false)
    let small = p <= 3_037_000_499
    unchecked:
        for i in 0..need: fa[i] = if small: fa[i] * fb[i] % p else: mul_mod(fa[i], fb[i], p)
    ntt_raw(mut fa, p, g, true)
    if not cyclic:
        if fa.len > n_out: fa.truncate(n_out)
        return fa
    if need == na: return fa
    var out = List.filled(na, 0)
    unchecked:
        for i in 0..na + nb - 1:
            let idx = i % na
            var s = out[idx] + fa[i]
            if s >= p: s -= p
            out[idx] = s
    out

fn ntt_rate2<P>(inverse: Bool) -> List<Mod<P>>:
    let rank2 = ctz(P - 1)
    let r = Mod<P>(pow_mod(root_of(P), (P - 1) >> rank2, P))
    let ir = r.inv
    var root = List.filled(rank2 + 1, Mod<P>(1))
    var iroot = List.filled(rank2 + 1, Mod<P>(1))
    var rate = List.filled(max(1, rank2), Mod<P>(1))
    unchecked:
        root[rank2] = if inverse: ir else: r
        iroot[rank2] = if inverse: r else: ir
        for i in (0..rank2).rev:
            root[i] = root[i + 1] * root[i + 1]
            iroot[i] = iroot[i + 1] * iroot[i + 1]
        var prod = Mod<P>(1)
        for i in 0..rank2 - 1:
            rate[i] = root[i + 2] * prod
            prod = prod * iroot[i + 2]
    rate

fn ntt_dif<P>(mut a: List<Mod<P>>):
    let n = a.len
    if n <= 1: return
    let h = ctz(n)
    let rate = ntt_rate2<P>(false)
    unchecked:
        for len in 0..h:
            let p = 1 << (h - len - 1)
            var rot = Mod<P>(1)
            let blocks = 1 << len
            for s in 0..blocks:
                let offset = s << (h - len)
                for i in 0..p:
                    let l = a[i + offset]
                    let r = a[i + offset + p] * rot
                    a[i + offset] = l + r
                    a[i + offset + p] = l - r
                if s + 1 != blocks: rot = rot * rate[ctz(s ^ -1)]

fn ntt_dit<P>(mut a: List<Mod<P>>):
    let n = a.len
    if n <= 1: return
    let h = ctz(n)
    let irate = ntt_rate2<P>(true)
    unchecked:
        for step in 0..h:
            let len = h - step
            let p = 1 << step
            var irot = Mod<P>(1)
            let blocks = 1 << (len - 1)
            for s in 0..blocks:
                let offset = s << (step + 1)
                for i in 0..p:
                    let l = a[i + offset]
                    let r = a[i + offset + p]
                    a[i + offset] = l + r
                    a[i + offset + p] = (l - r) * irot
                if s + 1 != blocks: irot = irot * irate[ctz(s ^ -1)]
        let inv_n = Mod<P>(n).inv
        for i in 0..n: a[i] = a[i] * inv_n

/// In-place number-theoretic transform of length `2^k` modulo an NTT-friendly prime `P`.
/// Set `inverse: true` for the inverse transform (includes division by `n`).
/// Set `bit_reversed: true` to stay in DIF/DIT bit-reversed frequency order (skips bit-reversal). O(n log n).
pub fn ntt<P>(mut a: List<Mod<P>>, inverse: Bool = false, bit_reversed: Bool = false):
    let n = a.len
    assert n.is_pow2, "ntt requires a power-of-two length"
    if n <= 1: return
    assert (P - 1) % n == 0, "modulus p - 1 must be divisible by n"
    if inverse:
        if not bit_reversed: bit_reverse(mut a)
        ntt_dit(mut a)
    else:
        ntt_dif(mut a)
        if not bit_reversed: bit_reverse(mut a)

/// Convolution of `a` and `b` over `Mod<P>`. O(n log n) via NTT (O(na * nb) when small).
/// With `cyclic: true`, requires `a.len == b.len` and wraps indices modulo `a.len`.
pub fn convolve<P>(a: List<Mod<P>>, b: List<Mod<P>>, cyclic: Bool = false) -> List<Mod<P>>:
    if a.is_empty or b.is_empty: return []
    let na = a.len
    let nb = b.len
    if cyclic:
        assert na == nb, "cyclic convolution requires equal-length inputs"
    let n_out = if cyclic: na else: na + nb - 1
    if min(na, nb) <= 60:
        var out = List.filled(n_out, Mod<P>(0))
        unchecked:
            for i in 0..na:
                let ai = a[i]
                if ai.v == 0: continue
                for j in 0..nb:
                    let idx = if cyclic: (i + j) % na else: i + j
                    out[idx] = out[idx] + ai * b[j]
        return out
    let need = if cyclic and na.is_pow2 and (P - 1) % na == 0: na else: bit_ceil(na + nb - 1)
    assert (P - 1) % need == 0, "modulus p - 1 must be divisible by n"
    var fa = List.filled(need, Mod<P>(0))
    var fb = List.filled(need, Mod<P>(0))
    unchecked:
        for i in 0..na: fa[i] = a[i]
        for i in 0..nb: fb[i] = b[i]
    ntt_dif(mut fa)
    ntt_dif(mut fb)
    unchecked:
        for i in 0..need: fa[i] = fa[i] * fb[i]
    ntt_dit(mut fa)
    if not cyclic:
        if fa.len > n_out: fa.truncate(n_out)
        return fa
    if need == na: return fa
    var out = List.filled(na, Mod<P>(0))
    unchecked:
        for i in 0..na + nb - 1:
            let idx = i % na
            out[idx] = out[idx] + fa[i]
    out

/// Convolution of integer lists modulo an arbitrary modulus `m >= 1` via 3-prime NTT + Garner CRT.
/// Exact for `m <= 10^9 + 7` and `a.len + b.len <= 2^23`. O(n log n).
pub fn convolve_mod(a: List<Int>, b: List<Int>, m: Int, cyclic: Bool = false) -> List<Int>:
    assert m >= 1
    if a.is_empty or b.is_empty: return []
    let na = a.len
    let nb = b.len
    if cyclic:
        assert na == nb, "cyclic convolution requires equal-length inputs"
    let n_out = if cyclic: na else: na + nb - 1
    if m == 1: return List.filled(n_out, 0)
    let ra = [x % m for x in a]
    let rb = [x % m for x in b]
    if min(na, nb) <= 60:
        var out = List.filled(n_out, 0)
        for i in 0..na:
            let ai = ra[i]
            if ai == 0: continue
            for j in 0..nb:
                let idx = if cyclic: (i + j) % na else: i + j
                out[idx] = (out[idx] + mul_mod(ai, rb[j], m)) % m
        return out
    let c1 = convolve_prime(ra, rb, P1, cyclic)
    let c2 = convolve_prime(ra, rb, P2, cyclic)
    let c3 = convolve_prime(ra, rb, P3, cyclic)
    let inv12 = pow_mod(P1 % P2, P2 - 2, P2)
    let inv12_3 = pow_mod((P1 % P3) * (P2 % P3) % P3, P3 - 2, P3)
    let p1_m = P1 % m
    let p12_m = mul_mod(p1_m, P2 % m, m)
    var out = List.filled(n_out, 0)
    for i in 0..n_out:
        let v1 = c1[i]
        let t2 = (c2[i] - v1 % P2 + P2) % P2 * inv12 % P2
        let v12_p3 = (v1 + t2 * P1) % P3
        let t3 = (c3[i] - v12_p3 + P3) % P3 * inv12_3 % P3
        out[i] = (v1 % m + mul_mod(t2 % m, p1_m, m) + mul_mod(t3 % m, p12_m, m)) % m
    out

/// Exact integer convolution (supports negative values) via 3-prime NTT + Garner CRT.
/// Exact whenever every output coefficient fits in `Int`. O(n log n).
pub fn convolve_int(a: List<Int>, b: List<Int>, cyclic: Bool = false) -> List<Int>:
    if a.is_empty or b.is_empty: return []
    let na = a.len
    let nb = b.len
    if cyclic:
        assert na == nb, "cyclic convolution requires equal-length inputs"
    let n_out = if cyclic: na else: na + nb - 1
    if min(na, nb) <= 60:
        var out = List.filled(n_out, 0)
        for i in 0..na:
            let ai = a[i]
            if ai == 0: continue
            for j in 0..nb:
                let idx = if cyclic: (i + j) % na else: i + j
                out[idx] += ai * b[j]
        return out
    let c1 = convolve_prime(a, b, P1, cyclic)
    let c2 = convolve_prime(a, b, P2, cyclic)
    let c3 = convolve_prime(a, b, P3, cyclic)
    let inv12 = pow_mod(P1 % P2, P2 - 2, P2)
    let inv12_3 = pow_mod((P1 % P3) * (P2 % P3) % P3, P3 - 2, P3)
    let p1_128 = I128(P1)
    let p12_128 = p1_128 * I128(P2)
    let total_m = p12_128 * I128(P3)
    let half_m = total_m / I128(2)
    var out = List.filled(n_out, 0)
    for i in 0..n_out:
        let v1 = c1[i]
        let t2 = (c2[i] - v1 % P2 + P2) % P2 * inv12 % P2
        let v12_p3 = (v1 + t2 * P1) % P3
        let t3 = (c3[i] - v12_p3 + P3) % P3 * inv12_3 % P3
        var val = I128(v1) + I128(t2) * p1_128 + I128(t3) * p12_128
        if val > half_m: val -= total_m
        out[i] = Int(val)
    out

/// Direction of a sum-over-subsets (SOS) zeta/Mobius transform.
pub enum SosOver = Sub | Sup

/// In-place subset-sum (`over: Sub`, default) or superset-sum (`over: Sup`) zeta transform
/// (`inverse: false`) or Mobius inverse (`inverse: true`). Length must be a power of two `2^k`. O(k * 2^k).
pub fn sos<T>(mut a: List<T>, over: SosOver = Sub, inverse: Bool = false):
    let n = a.len
    assert n.is_pow2, "sos requires a power-of-two length"
    var bit = 1
    match over:
        Sub:
            while bit < n:
                for mask in 0..n:
                    if (mask & bit) != 0:
                        if inverse: a[mask] -= a[mask ^ bit]
                        else: a[mask] += a[mask ^ bit]
                bit <<= 1
        Sup:
            while bit < n:
                for mask in 0..n:
                    if (mask & bit) == 0:
                        if inverse: a[mask] -= a[mask | bit]
                        else: a[mask] += a[mask | bit]
                bit <<= 1

/// Bitwise OR convolution: `out[i | j] += a[i] * b[j]`. Lengths must be equal powers of two. O(n log n).
pub fn convolve_or<T>(a: List<T>, b: List<T>) -> List<T>:
    assert a.len == b.len and a.len.is_pow2, "convolve_or requires equal power-of-two lengths"
    var fa = [x for x in a]
    var fb = [x for x in b]
    sos(mut fa, over: Sub)
    sos(mut fb, over: Sub)
    for i in 0..fa.len: fa[i] = fa[i] * fb[i]
    sos(mut fa, over: Sub, inverse: true)
    fa

/// Bitwise AND convolution: `out[i & j] += a[i] * b[j]`. Lengths must be equal powers of two. O(n log n).
pub fn convolve_and<T>(a: List<T>, b: List<T>) -> List<T>:
    assert a.len == b.len and a.len.is_pow2, "convolve_and requires equal power-of-two lengths"
    var fa = [x for x in a]
    var fb = [x for x in b]
    sos(mut fa, over: Sup)
    sos(mut fb, over: Sup)
    for i in 0..fa.len: fa[i] = fa[i] * fb[i]
    sos(mut fa, over: Sup, inverse: true)
    fa

fn fwht_xor(mut a: List<Int>):
    let n = a.len
    var step = 1
    while step < n:
        var i = 0
        while i < n:
            for j in i..i + step:
                let u = a[j]
                let v = a[j + step]
                a[j] = u + v
                a[j + step] = u - v
            i += step << 1
        step <<= 1

/// Bitwise XOR convolution: `out[i ^ j] += a[i] * b[j]`. Lengths must be equal powers of two. O(n log n).
pub fn convolve_xor(a: List<Int>, b: List<Int>) -> List<Int>:
    assert a.len == b.len and a.len.is_pow2, "convolve_xor requires equal power-of-two lengths"
    let n = a.len
    var fa = [x for x in a]
    var fb = [x for x in b]
    fwht_xor(mut fa)
    fwht_xor(mut fb)
    for i in 0..n: fa[i] *= fb[i]
    fwht_xor(mut fa)
    for i in 0..n: fa[i] /= n
    fa

@test
fn ntt_round_trip_and_convolve():
    type Z = Mod<998_244_353>
    var a = [Z(i * 7 + 3) for i in 0..16]
    let orig = [x for x in a]
    ntt(mut a)
    ntt(mut a, inverse: true)
    assert a == orig
    ntt(mut a, bit_reversed: true)
    ntt(mut a, inverse: true, bit_reversed: true)
    assert a == orig
    // Linear and cyclic convolution vs brute force (both small and NTT paths)
    for n in [5, 70]:
        let p = [Z(rand(0, 1000)) for _ in 0..n]
        let q = [Z(rand(0, 1000)) for _ in 0..n]
        let lin = convolve(p, q)
        var want_lin = List.filled(2 * n - 1, Z.zero)
        for i in 0..n:
            for j in 0..n: want_lin[i + j] = want_lin[i + j] + p[i] * q[j]
        assert lin == want_lin
        let cyc = convolve(p, q, cyclic: true)
        var want_cyc = List.filled(n, Z.zero)
        for i in 0..n:
            for j in 0..n: want_cyc[(i + j) % n] = want_cyc[(i + j) % n] + p[i] * q[j]
        assert cyc == want_cyc

@test
fn convolve_mod_and_int_match_brute_force():
    for n in [6, 65]:
        let a = [rand(-500, 500) for _ in 0..n]
        let b = [rand(-500, 500) for _ in 0..n]
        let got_int = convolve_int(a, b)
        var want_int = List.filled(2 * n - 1, 0)
        for i in 0..n:
            for j in 0..n: want_int[i + j] += a[i] * b[j]
        assert got_int == want_int
        let got_cyc = convolve_int(a, b, cyclic: true)
        var want_cyc = List.filled(n, 0)
        for i in 0..n:
            for j in 0..n: want_cyc[(i + j) % n] += a[i] * b[j]
        assert got_cyc == want_cyc
        let m = 1_000_000_007
        let got_mod = convolve_mod(a, b, m)
        assert got_mod == [x % m for x in want_int]
        let got_mod_cyc = convolve_mod(a, b, m, cyclic: true)
        assert got_mod_cyc == [x % m for x in want_cyc]

@test
fn sos_and_bitwise_convolutions():
    for bits in 0..=4:
        let n = 1 << bits
        let a = [rand(-20, 20) for _ in 0..n]
        let b = [rand(-20, 20) for _ in 0..n]
        var s = [x for x in a]
        sos(mut s)
        for mask in 0..n: assert s[mask] == sum(a[sub] for sub in 0..n if (sub & mask) == sub)
        sos(mut s, inverse: true)
        assert s == a
        var u = [x for x in a]
        sos(mut u, over: Sup)
        for mask in 0..n: assert u[mask] == sum(a[sup] for sup in 0..n if (sup & mask) == mask)
        sos(mut u, over: Sup, inverse: true)
        assert u == a
        var want_or = List.filled(n, 0)
        var want_and = List.filled(n, 0)
        var want_xor = List.filled(n, 0)
        for i in 0..n:
            for j in 0..n:
                want_or[i | j] += a[i] * b[j]
                want_and[i & j] += a[i] * b[j]
                want_xor[i ^ j] += a[i] * b[j]
        assert convolve_or(a, b) == want_or
        assert convolve_and(a, b) == want_and
        assert convolve_xor(a, b) == want_xor
    // Mod<P> works with sos_sub and convolve_or too
    type Z = Mod<998_244_353>
    let ma = [Z(1), Z(2), Z(3), Z(4)]
    let mb = [Z(5), Z(6), Z(7), Z(8)]
    assert convolve_or(ma, mb) == [Z(5), Z(28), Z(43), Z(184)]
