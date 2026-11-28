//! Number-theoretic transform, exact and modular convolution, SOS, and bitwise transforms.
//!
//! start: `convolve`, `ntt`, `sos`, `fwht`, `convolve_all`
//! example: `convolve([1, 2], [3, 4]) == [3, 10, 8]`
//!
//! Polynomials are coefficient lists, lowest degree first (`p[k]` is the coefficient of `x^k`).
//! There is one `convolve`: `on:` says how indices combine and `over:` which semiring the values
//! live in; the value type picks the algorithm (NTT for an NTT-prime `Mod<P>`, three primes and CRT
//! for any other `Mod` or `Int`, FFT for `F64`), and `cpl explain` names it. `cyclic: Bool = false`:
//! - linear (`cyclic: false`): length `a.len + b.len - 1` (or `[]` if either input is empty)
//! - cyclic (`cyclic: true`): requires `a.len == b.len`, length `n = a.len`, index `(i + j) % n`.
use std.algebra.{Monoid, Semiring}
use std.math.{abs, acos, cos, sin, is_pow2, primitive_root, bit, gcd, lcm, isqrt, popcount, trunc, bit_ceil, ctz}
use std.mod.{Mod}
use std.seq.{collect, all}

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

/// In-place number-theoretic transform of `a` modulo the NTT-friendly prime `P`.
///
/// The forward transform maps `a` to `â[k] = sum_j a[j] * w^(j k)` with `w =
/// primitive_root(P)^((P - 1) / n)`, in natural order. `inverse: true` uses `w^-1` and divides
/// by `n`, so it exactly undoes the forward transform. `bit_reversed: true` skips the
/// bit-reversal permutation: the forward transform then leaves `â` in bit-reversed order and the
/// inverse expects its input in that order, so a pointwise product between the two needs no
/// reordering. Radix-2 butterflies over one twiddle table per call; `cpl explain` names the
/// engine.
///
/// cost: \(O(n \log n)\)
/// empty: `var a: List<Mod<17>> = []; ntt(mut a); a == []` (traps R0401)
/// order: in place; natural frequency order unless `bit_reversed: true`
/// traps: when `a.len` is not a power of two dividing `P - 1`, or `P` is not prime
/// example: `type Z = Mod<17>; var a = [Z(1), Z(0), Z(0), Z(0)]; ntt(mut a); a == [Z(1), Z(1), Z(1), Z(1)]`
/// example: `type Z = Mod<17>; var a = [Z(1), Z(2), Z(3), Z(4)]; ntt(mut a); ntt(mut a, inverse: true); a == [Z(1), Z(2), Z(3), Z(4)]`
/// example: `type Z = Mod<17>; var a = [Z(5), Z(6)]; ntt(mut a, bit_reversed: true); ntt(mut a, inverse: true, bit_reversed: true); a == [Z(5), Z(6)]`
pub fn ntt<P>(mut a: List<Mod<P>>, inverse: Bool = false, bit_reversed: Bool = false):
    let n = a.len
    assert n.is_pow2, "ntt requires a power-of-two length"
    if n <= 1: return
    assert (P - 1) % n == 0, "modulus p - 1 must be divisible by n"
    var g = Mod<P>(3)
    if P == 754_974_721: g = Mod<P>(11)
    elif P != NTT_P1 and P != NTT_P3 and P != NTT_P2: g = Mod<P>(primitive_root(P))
    // tw[s] is block s's twiddle in every layer: `z^bitrev(s)` over `log2(n) - 1` bits, `z` the
    // primitive n-th root (conjugated for the inverse). One table, filled by independent products,
    // keeps a serial twiddle chain off the short-block layers.
    let h = ctz(n)
    let half = n >> 1
    var roots = List.filled(h + 1, Mod<P>(1))
    roots[h] = g ** (if inverse: P - 1 - (P - 1) / n else: (P - 1) / n)
    var tw = List.filled(half, Mod<P>(1))
    if inverse and not bit_reversed: bit_reverse(mut a)
    unchecked:
        for k in (0..h).rev: roots[k] = roots[k + 1] * roots[k + 1]
        var b = 1
        while b < half:
            let c = roots[ctz(b) + 2]
            for t in 0..b: tw[b + t] = tw[t] * c
            b *= 2
        var p = if inverse: 1 else: half
        while 0 < p < n:
            var o = 0
            var s = 0
            while o < n:
                let rot = tw[s]
                if inverse:
                    for i in o..o + p:
                        let (l, r) = (a[i], a[i + p])
                        a[i] = l + r
                        a[i + p] = (l - r) * rot
                else:
                    for i in o..o + p:
                        let (l, r) = (a[i], a[i + p] * rot)
                        a[i] = l + r
                        a[i + p] = l - r
                o += 2 * p
                s += 1
            p = if inverse: p * 2 else: p >> 1
        if inverse:
            let inv_n = Mod<P>(n).inv()
            for i in 0..n: a[i] *= inv_n
    if not inverse and not bit_reversed: bit_reverse(mut a)

/// Which arguments of a convolution over a semiring other than `(+, *)` have the shape that makes
/// it fast: convex for `min`, concave for `max`. `Neither` asks for the \(O(n m)\) loop by name.
pub enum Curved = Both | Right | Neither

/// A custom index op for `convolve(on:)`: `a[i] * b[j]` lands at `at(i, j)`, or nowhere when it
/// is `None`; the output has `len(a.len, b.len)` entries. User libraries implement it the same way.
pub trait IndexOp:
    fn len(self, na: Int, nb: Int) -> Int
    fn at(self, i: Int, j: Int) -> ?Int

/// The index op of a subset convolution: `i | j` over disjoint pairs `i & j == 0`.
pub struct Disjoint

impl IndexOp for Disjoint:
    fn len(self, na: Int, nb: Int) -> Int:
        assert na == nb and na.is_pow2, "a subset convolution needs equal power-of-two lengths"
        na
    fn at(self, i: Int, j: Int) -> ?Int = if i & j == 0: Some(i | j) else: None

/// `on: subset` in `convolve`.
pub const subset = Disjoint()

/// The convolution of `a` and `b`: `c[on(i, j)]` is the `over.add` of every `over.mul(a[i], b[j])`.
///
/// The default is the polynomial product. The value type picks the algorithm and `cpl explain`
/// names it: NTT for an NTT-prime `Mod<P>`, three primes and CRT for any other `Mod` (runtime
/// moduli too) or `Int`, FFT for `F64`. Output lengths:
/// - linear (default): `a.len + b.len - 1`, or `[]` when either input is empty;
/// - `cyclic: true`: `a.len == b.len == n`, length `n`, index `(i + j) % n`;
/// - `on: ^`, `|`, `&` and `subset`: `a.len == b.len == 2^k`, length `2^k`; `subset` sums only
///   pairs with `i & j == 0`;
/// - `on: gcd` and `on: lcm`: index from 1 (position 0 contributes nothing and `c[0]` is zero),
///   length `max(a.len, b.len)`; `lcm` values at or past that length are dropped.
/// - any other `IndexOp` value: length `on.len(a.len, b.len)`, `a[i] * b[j]` lands at `on.at(i, j)`.
///
/// Any `over:` but `Semiring(+, *)` needs `shape:` (`Both`: both inputs convex for `min`, concave
/// for `max`, slopes merged; `Right`: only `b`, `monotone_minima`; `Neither`: the \(O(n m)\) loop by
/// name). Leaving `shape:` out there, or giving it with `Semiring(+, *)`, is a compile error.
///
/// cost: \(O(n \log n)\) bitwise and for the product over a [`FastProduct`] type (NTT for `Mod<P>` when `P - 1` has the power of two, and in halves of it when `P - 1` has 2^20; four complex FFTs over sqrt(P)-split halves for any other `Mod<P>` below 2^31 while `n log2 n * P` is under 8.6e14, else three NTT primes and Garner's CRT while `(P - 1)^2 * min(n, m)` is under 7.8e25, and for `Int` while `max |a| * max |b| * min(n, m)` is under 1e25; a complex FFT for `F64`), the cyclic form included; \(O(n + m)\) `Both`; \(O(n m)\) today for any other product, `subset`, `gcd`/`lcm`, `Right`, `Neither` and any `IndexOp` (the transforms are planned)
/// empty: `let e: List<Int> = []; convolve([1, 2], e) == [] and convolve(e, [1, 2]) == []`
/// order: by output index `0..len`; a new `List`, keyed from 0
/// traps: when `cyclic: true` and `a.len != b.len`; when a bitwise or `subset` input is not a power-of-two length equal to the other's; when an `Int` output overflows; never for `F64`, whose FFT outputs are each off from the exact sum by up to about `(sum of a[i]^2 + sum of b[j]^2) * log2(n) * 2^-52` (inputs up to 60 long take the exact loop)
/// example: `convolve([1, 2], [3, 4]) == [3, 10, 8]`
/// example: `convolve([0.5, 2.0], [4.0, 1.0]) == [2.0, 8.5, 2.0]`
/// example: `convolve([1, 2, 3], [4, 5, 6], cyclic: true) == [31, 31, 28]`
/// example: `convolve([1, 2], [3, 4], on: ^) == [11, 10]`
/// example: `convolve([1, 2, 3, 4], [1, 1, 1, 1], on: subset) == [1, 3, 4, 10]`
/// example: `convolve([0, 1, 1, 1], [0, 1, 1, 1], on: gcd) == [0, 7, 1, 1]`
/// example: `convolve([0, 1, 1, 1], [0, 1, 1, 1], on: lcm) == [0, 1, 3, 3]`
/// example: `convolve([0, 1, 3], [0, 2, 5], over: Semiring(min, +), shape: Both) == [0, 1, 3, 5, 8]`
/// example: `let e: List<Int> = []; convolve([1, 2], e) == []`
pub fn convolve<T: Add + Sub + Mul + Zero, O, R>(
    a: List<T>,
    b: List<T>,
    on: O = +,
    over: R = Semiring(+, *),
    shape: ?Curved = None,
    cyclic: Bool = false,
) -> List<T> where T: One + Ord:
    let na = a.len
    let nb = b.len
    if cyclic:
        assert na == nb, "cyclic convolution requires equal-length inputs"
    if not (same(over.add, +) and same(over.mul, *)):
        return semiring_convolve(a, b, over, shape)
    if same(on, ^) or same(on, |) or same(on, &):
        assert na == nb and na.is_pow2, "a bitwise convolution needs equal power-of-two lengths"
        var fa = a.clone()
        var fb = b.clone()
        if same(on, ^):
            fwht(mut fa)
            fwht(mut fb)
        elif same(on, |):
            sos(mut fa)
            sos(mut fb)
        else:
            sos(mut fa, over: Sup)
            sos(mut fb, over: Sup)
        for i in 0..na: fa[i] *= fb[i]
        if same(on, ^): fwht(mut fa, inverse: true)
        elif same(on, |): sos(mut fa, inverse: true)
        else: sos(mut fa, over: Sup, inverse: true)
        return fa
    if na == 0 or nb == 0: return []
    if T: FastProduct:
        if same(on, +):
            if let c = T.product(a, b):
                if not cyclic: return c
                return [if k + na < c.len: c[k] + c[k + na] else: c[k] for k in 0..na]
    let by_divisor = same(on, gcd) or same(on, lcm)
    var n_out = if by_divisor: max(na, nb) elif cyclic: na else: na + nb - 1
    if O: IndexOp:
        n_out = on.len(na, nb)
    var out = List.filled(n_out, T.zero)
    for i in 0..na:
        for j in 0..nb:
            var at: ?Int = None
            if O: IndexOp:
                at = on.at(i, j)
            elif not by_divisor or (i > 0 and j > 0):
                at = on(i, j)
            if let k0 = at:
                let k = if cyclic and k0 >= 0: k0 % na else: k0
                if 0 <= k < n_out:
                    out[k] = out[k] + a[i] * b[j]
    out

fn same<A, B>(a: A, b: B) -> Bool = a == b

/// A number type with a fast linear product, which `convolve` uses for its default `on: +`,
/// `over: Semiring(+, *)` form. `Mod<P>` has one for every `P` and `F64` has one (see the cost line); a user number type joins by implementing
/// it the same way.
pub trait FastProduct:
    /// `convolve(a, b)` for nonempty `a` and `b`, or `None` when this type has no fast product for
    /// these lengths (then `convolve` runs its \(O(n m)\) loop).
    fn product(a: List<Self>, b: List<Self>) -> ?List<Self>

impl FastProduct for Mod<P>:
    fn product(a: List<Mod<P>>, b: List<Mod<P>>) -> ?List<Mod<P>>:
        let (na, nb) = (a.len, b.len)
        let n_out = na + nb - 1
        if min(na, nb) <= 60:
            var out = List.filled(n_out, Mod<P>(0))
            unchecked:
                for i in 0..na:
                    let ai = a[i]
                    if ai.v == 0: continue
                    for j in 0..nb: out[i + j] += ai * b[j]
            return Some(out)
        if let got = ntt_product(a, b): return Some(got)
        // A `P - 1` with 2^20 in it is an NTT prime's: a product past its power of two is two of
        // half the length, and the branch folds on `P`, so its C++ has no FFT or CRT.
        if (P - 1) % (1 << 20) == 0: return Some(halves_product(a, b))
        split_product(a, b) ?? crt_product(a, b)

impl FastProduct for Int:
    /// Two NTT primes and CRT while `max |a| * max |b| * min(a.len, b.len)` stays under 8e16
    /// (their product is about 1.7e17), three under 1e25 (about 7.9e25); past that it is `None`
    /// and `convolve` runs its loop, which traps on an overflow.
    fn product(a: List<Int>, b: List<Int>) -> ?List<Int>:
        let (na, nb) = (a.len, b.len)
        if min(na, nb) <= 60: return None
        var (ma, mb) = (0, 0)
        for x in a: ma max= abs(x)
        for x in b: mb max= abs(x)
        let bound = I128(ma) * I128(mb) * I128(min(na, nb))
        if bound >= I128(10_000_000_000_000) * I128(1_000_000_000_000): return None
        let three = bound >= I128(80_000_000_000_000_000)
        let r1 = residues(Mod<NTT_P1>(0), a, b) ?? return None
        let r2 = residues(Mod<NTT_P2>(0), a, b) ?? return None
        var r3: List<Mod<NTT_P3>> = []
        if three: r3 = residues(Mod<NTT_P3>(0), a, b) ?? return None
        var out = List.filled(na + nb - 1, 0)
        for k in 0..out.len:
            let c = garner2(r1[k], r2[k])
            if not three:
                out[k] = if c > P12 / 2: c - P12 else: c
                continue
            let w = garner3(c, r3[k])
            out[k] = Int(if w * I128(2) > I128(P12) * I128(NTT_P3): w - I128(P12) * I128(NTT_P3) else: w)
        Some(out)

const P12 = 167_477_612_308_856_833
const NTT_P1 = 998_244_353
const NTT_P2 = 167_772_161
const NTT_P3 = 469_762_049

// `a * b` modulo `P` by its NTT; `None` when `P - 1` lacks the product's power of two.
fn residues<P>(_: Mod<P>, a: List<Int>, b: List<Int>) -> ?List<Mod<P>> =
    ntt_product([Mod<P>(x) for x in a], [Mod<P>(x) for x in b])

fn ntt_product<P>(a: List<Mod<P>>, b: List<Mod<P>>) -> ?List<Mod<P>>:
    let (na, nb) = (a.len, b.len)
    let n_out = na + nb - 1
    let need = bit_ceil(n_out)
    if (P - 1) % need != 0: return None
    var fa = List.filled(need, Mod<P>(0))
    var fb = List.filled(need, Mod<P>(0))
    unchecked:
        for i in 0..na: fa[i] = a[i]
        for i in 0..nb: fb[i] = b[i]
    ntt(mut fa, bit_reversed: true)
    ntt(mut fb, bit_reversed: true)
    unchecked:
        for i in 0..need: fa[i] *= fb[i]
    ntt(mut fa, inverse: true, bit_reversed: true)
    fa.truncate(n_out)
    Some(fa)

// Garner over p1 = 998_244_353, p2 = 167_772_161, p3 = 469_762_049: 47_450_712 is 1 / p1 mod p2,
// 115_990_628 is 1 / (p1 p2) mod p3, and P12 is p1 p2. `garner2` is the value mod p1 p2 and
// `garner3` the value mod p1 p2 p3, each in `0..` its modulus.
fn garner2(r1: Mod<NTT_P1>, r2: Mod<NTT_P2>) -> Int:
    let x1 = Int(r1.v)
    x1 + Int(((r2 - Mod<NTT_P2>(x1)) * Mod<NTT_P2>(47_450_712)).v) * NTT_P1

fn garner3(c: Int, r3: Mod<NTT_P3>) -> I128 = I128(c) + I128(garner_digit(c, r3)) * I128(P12)

fn garner_digit(c: Int, r3: Mod<NTT_P3>) -> Int = Int(((r3 - Mod<NTT_P3>(c)) * Mod<NTT_P3>(115_990_628)).v)

// `a * b` for a `Mod<P>` whose NTT can't take the whole product: the longer side's halves, each
// times the other side, added at their offsets.
fn halves_product<P>(a: List<Mod<P>>, b: List<Mod<P>>) -> List<Mod<P>>:
    if a.len < b.len: return halves_product(b, a)
    let h = a.len / 2
    var out = List.filled(a.len + b.len - 1, Mod<P>(0))
    for (lo, hi) in [(0, h), (h, a.len)]:
        let got = Mod<P>.product([a[i] for i in lo..hi], b)!
        unchecked:
            for i in 0..got.len: out[lo + i] += got[i]
    out

// `a * b` for a `Mod<P>` whose `P - 1` lacks the power of two: the exact product of the residues,
// Splits each value as hi * cut + lo with cut about sqrt(P), packs hi + i lo into one complex
// list per input and gets the four cross products from four FFTs (KACTL's convMod). `None` when
// P >= 2^31 or n log2 n * P reaches 8.6e14, where a coefficient could round wrong.
fn split_product<P>(a: List<Mod<P>>, b: List<Mod<P>>) -> ?List<Mod<P>>:
    let n_out = a.len + b.len - 1
    let n = bit_ceil(n_out)
    var lg = 0
    while (1 << lg) < n: lg += 1
    if P >= 1 << 31 or F64(n) * F64(lg) * F64(P) >= 8.6e14: return None
    let cut = P.isqrt + 1
    var lr = List.filled(n, 0.0)
    var li = List.filled(n, 0.0)
    var rr = List.filled(n, 0.0)
    var ri = List.filled(n, 0.0)
    unchecked:
        for i in 0..a.len:
            let x = Int(a[i].v)
            lr[i] = F64(x / cut)
            li[i] = F64(x % cut)
        for i in 0..b.len:
            let x = Int(b[i].v)
            rr[i] = F64(x / cut)
            ri[i] = F64(x % cut)
    fft(mut lr, mut li)
    fft(mut rr, mut ri)
    var olr = List.filled(n, 0.0)
    var oli = List.filled(n, 0.0)
    var osr = List.filled(n, 0.0)
    var osi = List.filled(n, 0.0)
    let s = 2.0 * F64(n)
    unchecked:
        for i in 0..n:
            let j = (n - i) & (n - 1)
            let (pr, pi) = (lr[i] + lr[j], li[i] - li[j])
            let (qr, qi) = (lr[i] - lr[j], li[i] + li[j])
            let (xr, xi) = (rr[i], ri[i])
            olr[j] = (pr * xr - pi * xi) / s
            oli[j] = (pr * xi + pi * xr) / s
            osr[j] = (qr * xi + qi * xr) / s
            osi[j] = -(qr * xr - qi * xi) / s
    fft(mut olr, mut oli)
    fft(mut osr, mut osi)
    // Each sum is a non-negative integer within 0.5 of the float, so trunc(x + 0.5) rounds it.
    // hh < n cut^2 and hl, ll < 2 n cut^2, so (hh % P) cut^2 % P + hl cut + ll stays under 2^63.
    let c2 = cut * cut % P
    var out = List.filled(n_out, Mod<P>(0))
    unchecked:
        for k in 0..n_out:
            let hh = trunc(olr[k] + 0.5) % P
            let hl = trunc(oli[k] + 0.5) + trunc(osr[k] + 0.5)
            out[k] = Mod<P>(hh * c2 + hl * cut + trunc(osi[k] + 0.5))
    Some(out)

// modulo the three NTT primes, rebuilt by Garner and reduced mod `P`. `None` when a coefficient
// could reach p1 p2 p3 (about 7.9e25), past which the residues no longer pin it down.
fn crt_product<P>(a: List<Mod<P>>, b: List<Mod<P>>) -> ?List<Mod<P>>:
    let bound = I128(P - 1) * I128(P - 1) * I128(min(a.len, b.len))
    if bound >= I128(P12) * I128(NTT_P3): return None
    let xa = [Int(x.v) for x in a]
    let xb = [Int(x.v) for x in b]
    let r1 = residues(Mod<NTT_P1>(0), xa, xb) ?? return None
    let r2 = residues(Mod<NTT_P2>(0), xa, xb) ?? return None
    let r3 = residues(Mod<NTT_P3>(0), xa, xb) ?? return None
    let p12 = Mod<P>(P12)
    var out = List.filled(r1.len, Mod<P>(0))
    unchecked:
        for k in 0..out.len:
            let c = garner2(r1[k], r2[k])
            out[k] = Mod<P>(c) + Mod<P>(garner_digit(c, r3[k])) * p12
    Some(out)

impl FastProduct for F64:
    /// The FFT product: `a + i b` goes through one complex transform, is squared and split back,
    /// then one more transform, \(O(n \log n)\). `None` for `min(a.len, b.len) <= 60`, where
    /// `convolve` runs its exact loop. Each output is off from the exact sum by at most about
    /// `(sum of a[i]^2 + sum of b[j]^2) * log2(n) * 2^-52`, so integer inputs round back exactly
    /// while that stays under `0.5`.
    fn product(a: List<F64>, b: List<F64>) -> ?List<F64>:
        let (na, nb) = (a.len, b.len)
        if min(na, nb) <= 60: return None
        let n_out = na + nb - 1
        let n = bit_ceil(n_out)
        var re = List.filled(n, 0.0)
        var im = List.filled(n, 0.0)
        unchecked:
            for i in 0..na: re[i] = a[i]
            for i in 0..nb: im[i] = b[i]
        fft(mut re, mut im)
        var ore = List.filled(n, 0.0)
        var oim = List.filled(n, 0.0)
        unchecked:
            for i in 0..n:
                let j = (n - i) & (n - 1)
                let (xr, xi) = (re[i], im[i])
                let (yr, yi) = (re[j], im[j])
                // (in[j])^2 - conj((in[i])^2)
                ore[i] = (yr * yr - yi * yi) - (xr * xr - xi * xi)
                oim[i] = 2.0 * yr * yi + 2.0 * xr * xi
        fft(mut ore, mut oim)
        let scale = 4.0 * F64(n)
        Some([oim[i] / scale for i in 0..n_out])

// In-place forward complex FFT of `re + i im`, length a power of two; the roots of each level are
// taken from `cos`/`sin` directly so their error doesn't build up across levels.
fn fft(mut re: List<F64>, mut im: List<F64>):
    let n = re.len
    bit_reverse(mut re)
    bit_reverse(mut im)
    let pi = acos(-1.0)
    // rt[k + j] = e^(i pi j / k): each level's odd entries are the level above times one root,
    // so a transform takes log n trig calls, not n.
    var wr = List.filled(max(n, 2), 1.0)
    var wi = List.filled(max(n, 2), 0.0)
    var k = 2
    while k < n:
        let (xr, xi) = (cos(pi / F64(k)), sin(pi / F64(k)))
        unchecked:
            for i in k..2 * k:
                let (ar, ai) = (wr[i / 2], wi[i / 2])
                if i % 2 == 0:
                    wr[i] = ar
                    wi[i] = ai
                else:
                    wr[i] = ar * xr - ai * xi
                    wi[i] = ar * xi + ai * xr
        k *= 2
    unchecked:
        var i = 0
        while i + 1 < n:
            let (xr, xi) = (re[i + 1], im[i + 1])
            re[i + 1] = re[i] - xr
            im[i + 1] = im[i] - xi
            re[i] += xr
            im[i] += xi
            i += 2
    k = 2
    while k < n:
        unchecked:
            var i = 0
            while i < n:
                for j in 0..k:
                    let (xr, xi) = (re[i + j + k], im[i + j + k])
                    let (cr, ci) = (wr[j + k], wi[j + k])
                    let zr = cr * xr - ci * xi
                    let zi = cr * xi + ci * xr
                    re[i + j + k] = re[i + j] - zr
                    im[i + j + k] = im[i + j] - zi
                    re[i + j] += zr
                    im[i + j] += zi
                i += 2 * k
        k *= 2

/// `convolve` over another semiring: `out[i + j]` folds `over.mul(a[i], b[j])` with `over.add`.
/// `shape: Both` merges slopes (both inputs convex for `min`, concave for `max`, `mul` is `+`).
fn semiring_convolve<T: Add + Sub + Ord, R>(a: List<T>, b: List<T>, over: R, shape: ?Curved) -> List<T>:
    if a.is_empty or b.is_empty: return []
    let add = Monoid<T>.of(over.add)
    let mul = Monoid<T>.of(over.mul)
    if shape == Some(Curved.Both):
        assert same(over.mul, +), "`shape: Both` needs `mul` to be `+`"
        let low = same(over.add, min)
        var out = [a[0] + b[0]]
        var i = 0
        var j = 0
        while i + 1 < a.len or j + 1 < b.len:
            let take_a = if j + 1 >= b.len: true elif i + 1 >= a.len: false elif low: a[i + 1] - a[i] <= b[j + 1] - b[j] else: a[i + 1] - a[i] >= b[j + 1] - b[j]
            let last = out.last!
            if take_a:
                out.push(last + (a[i + 1] - a[i]))
                i += 1
            else:
                out.push(last + (b[j + 1] - b[j]))
                j += 1
        return out
    var acc: List<?T> = List.filled(a.len + b.len - 1, None)
    for i in 0..a.len:
        for j in 0..b.len:
            let v = mul.op(a[i], b[j])
            acc[i + j] = if let c = acc[i + j]: Some(add.op(c, v)) else: Some(v)
    [x! for x in acc]

/// Direction of a sum-over-subsets (SOS) zeta/Mobius transform.
pub enum SosOver = Sub | Sup

/// In-place sum-over-subsets (zeta) transform of `a`, length `2^k`, or its Mobius inverse.
///
/// With `over: Sub` each `a[m]` becomes the sum of the old `a[s]` over the submasks `s` of `m`;
/// with `over: Sup`, over the supermasks. `inverse: true` is the Mobius transform, which undoes
/// the zeta transform of the same direction. Positions are zero-based masks, so any
/// `IndexedMut` of power-of-two length works.
///
/// cost: \(O(k 2^k)\)
/// empty: `var a: List<Int> = []; sos(mut a); a == []` (traps R0401)
/// order: in place, indexed by mask
/// traps: when `a.len` is not a power of two
/// example: `var a = [1, 2, 3, 4]; sos(mut a); a == [1, 3, 4, 10]`
/// example: `var a = [1, 2, 3, 4]; sos(mut a, over: Sup); a == [10, 6, 7, 4]`
/// example: `var a = [1, 3, 4, 10]; sos(mut a, inverse: true); a == [1, 2, 3, 4]`
pub fn sos<S: IndexedMut>(mut a: S, over: SosOver = Sub, inverse: Bool = false):
    let n = a.len
    let ks = a.keys
    assert n.is_pow2, "sos requires a power-of-two length"
    var bit = 1
    match over:
        Sub =>
            while bit < n:
                for mask in 0..n:
                    if (mask & bit) != 0:
                        if inverse: a[ks.nth(mask)] -= a[ks.nth(mask ^ bit)]
                        else: a[ks.nth(mask)] += a[ks.nth(mask ^ bit)]
                bit <<= 1
        Sup =>
            while bit < n:
                for mask in 0..n:
                    if (mask & bit) == 0:
                        if inverse: a[ks.nth(mask)] -= a[ks.nth(mask | bit)]
                        else: a[ks.nth(mask)] += a[ks.nth(mask | bit)]
                bit <<= 1

/// In-place Walsh-Hadamard (XOR) transform of `a`, length `2^k`.
///
/// The forward transform maps `a[m]` to `sum_s (-1)^popcount(m & s) * a[s]`. `inverse: true`
/// applies the same butterflies and divides by `n`, so it undoes the forward transform; for
/// `Int` the division is exact on any forward-transformed input. Works on any `IndexedMut`
/// (`List<T>`, `[D]T`, mutable slices).
///
/// cost: \(O(n \log n)\)
/// empty: `var a: List<Int> = []; fwht(mut a); a == []` (traps R0401)
/// order: in place, indexed by mask
/// traps: when `a.len` is not a power of two; with `inverse: true` on `Int`, when an entry is not divisible by `n`
/// example: `var a = [1, 2]; fwht(mut a); a == [3, -1]`
/// example: `var a = [1, 2, 3, 4]; fwht(mut a); a == [10, -2, -4, 0]`
/// example: `var a = [10, -2, -4, 0]; fwht(mut a, inverse: true); a == [1, 2, 3, 4]`
pub fn fwht<S: IndexedMut<D, T>, D: Discrete, T>(mut a: S, inverse: Bool = false) where T: One + Zero + Add + Div + Eq + Mul + Sub:
    let n = a.len
    let ks = a.keys
    assert n.is_pow2, "fwht requires a power-of-two length"
    var step = 1
    unchecked:
        while step < n:
            var i = 0
            while i < n:
                for j in i..i + step:
                    let u = a[ks.nth(j)]
                    let v = a[ks.nth(j + step)]
                    a[ks.nth(j)] = u + v
                    a[ks.nth(j + step)] = u - v
                i += step << 1
            step <<= 1
    if inverse and n > 1:
        var n_t = T.one
        var m = n
        while m > 1:
            n_t = n_t + n_t
            m >>= 1
        let inv_n = T.one / n_t
        unchecked:
            if inv_n != T.zero:
                for i in 0..n: a[ks.nth(i)] = a[ks.nth(i)] * inv_n
            else:
                for i in 0..n:
                    let q = a[ks.nth(i)] / n_t
                    assert q * n_t == a[ks.nth(i)], "`fwht(inverse: true)` needs every entry divisible by `n`"
                    a[ks.nth(i)] = q

/// \(O(\log^2 n)\) amortized per step, \(O(n \log^2 n)\) total. Online (relaxed) NTT convolution:
/// supply `(a[k], b[k])` one step at a time and immediately receive `c[k] = sum_{i=0..=k} a[i] * b[k - i]`.
pub struct RelaxedConvolve<P>:
    a: List<Mod<P>>
    b: List<Mod<P>>
    c: List<Mod<P>>

    /// An empty online convolution: no values pushed yet.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `type Z = Mod<17>; var rc = RelaxedConvolve<17>(); rc.push(Z(0), Z(0)) == Z(0)`
    pub fn new() -> RelaxedConvolve<P> = RelaxedConvolve([], [], [])

    /// Append `a[k] = ai` and `b[k] = bi` at the next index `k` and return `c[k] = sum_{i=0..=k}
    /// a[i] * b[k - i]`.
    ///
    /// `c[k]` depends only on values already pushed, so `a[k+1]` and `b[k+1]` may be computed from
    /// it: the shape of online DP like `f[k] = sum f[i] g[k - i]`.
    ///
    /// cost: \(O(\log^2 k)\) amortized
    /// traps: when `k` reaches the largest NTT length `P` supports
    /// example: `type Z = Mod<17>; var rc = RelaxedConvolve<17>(); rc.push(Z(1), Z(3)) == Z(3)`
    /// example: `type Z = Mod<17>; var rc = RelaxedConvolve<17>(); rc.push(Z(1), Z(3)); rc.push(Z(2), Z(4)) == Z(10)`
    pub fn push(mut self, ai: Mod<P>, bi: Mod<P>) -> Mod<P>:
        a.push(ai)
        b.push(bi)
        let i = a.len - 1
        while c.len <= i: c.push(Mod<P>(0))
        c[i] += a[0] * b[i]
        if i > 0: c[i] += a[i] * b[0]
        let q = i + 1
        var w = 1
        while q % w == 0 and q / w >= 2:
            let j = q / w - 1
            add_block(mut c, window(a, w, w), window(b, j * w, w), (j + 1) * w, q)
            if j != 1: add_block(mut c, window(b, w, w), window(a, j * w, w), (j + 1) * w, q)
            w *= 2
        c[i]

/// \(O(\log^2 n)\) amortized per step. Semi-relaxed NTT convolution against a fixed `b`
/// (supply `a[k]` online and immediately receive `c[k] = sum_{i=0..=k} a[i] * b[k - i]`).
pub struct SemiRelaxedConvolve<P>:
    b: List<Mod<P>>
    a: List<Mod<P>>
    c: List<Mod<P>>

    /// An online convolution against the fixed polynomial `b`, nothing pushed yet.
    ///
    /// `b` is zero-padded as far as the pushes go, so pushing past `b.len` is allowed; an empty `b`
    /// makes every `c[k]` zero.
    ///
    /// cost: \(O(b.\mathit{len})\)
    /// empty: `type Z = Mod<17>; let e: List<Z> = []; var sc = SemiRelaxedConvolve<17>(e); sc.push(Z(5)); sc.push(Z(3)) == Z(0)`
    /// traps: never
    /// example: `type Z = Mod<17>; var sc = SemiRelaxedConvolve<17>([Z(1), Z(2)]); sc.push(Z(1)) == Z(1)`
    /// example: `type Z = Mod<17>; let e: List<Z> = []; var sc = SemiRelaxedConvolve<17>(e); sc.push(Z(5)) == Z(0)`
    pub fn new(b: List<Mod<P>>) -> SemiRelaxedConvolve<P> = SemiRelaxedConvolve(b.clone(), [], [])

    /// Append `a[k] = ai` at the next index `k` and return `c[k] = sum_{i=0..=k} a[i] * b[k - i]`.
    ///
    /// cost: \(O(\log^2 k)\) amortized
    /// traps: when `k` reaches the largest NTT length `P` supports
    /// example: `type Z = Mod<17>; var sc = SemiRelaxedConvolve<17>([Z(1), Z(2)]); sc.push(Z(1)); sc.push(Z(2)); sc.push(Z(5)) == Z(9)`
    pub fn push(mut self, ai: Mod<P>) -> Mod<P>:
        a.push(ai)
        let i = a.len - 1
        while c.len <= i: c.push(Mod<P>(0))
        if not b.is_empty: c[i] += ai * b[0]
        let q = i + 1
        var w = 1
        while q % w == 0 and w < b.len:
            add_block(mut c, window(a, q - w, w), window(b, w, w), q, q)
            w *= 2
        c[i]

/// The product of every polynomial in `polys`, multiplied as a balanced tree.
///
/// The empty product is `[1]`. An empty polynomial among the factors is zero (as in `convolve`),
/// so the result is `[]`.
///
/// cost: \(O(S \log^2 S)\) for total length `S`
/// empty: `type Z = Mod<17>; let e: List<List<Z>> = []; convolve_all(e) == [Z(1)]`
/// order: lowest degree first; a new `List`, keyed from 0
/// traps: when the product outgrows the largest NTT length `P` supports
/// example: `type Z = Mod<17>; convolve_all([[Z(1), Z(1)], [Z(1), Z(1)]]) == [Z(1), Z(2), Z(1)]`
/// example: `type Z = Mod<17>; let e: List<List<Z>> = []; convolve_all(e) == [Z(1)]`
/// example: `type Z = Mod<17>; let e: List<Z> = []; convolve_all([[Z(1), Z(1)], e]) == []`
pub fn convolve_all<P>(polys: List<List<Mod<P>>>) -> List<Mod<P>>:
    if polys.is_empty: return [Mod<P>(1)]
    var layer = polys.clone()
    while layer.len > 1:
        var up = List<List<Mod<P>>>()
        var k = 0
        while k + 1 < layer.len:
            up.push(times(layer[k], layer[k + 1]))
            k += 2
        if k < layer.len: up.push(layer[k])
        layer = up
    layer[0]

/// The 2D product of `a` and `b`: `out[i][j]` sums `a[r][c] * b[i - r][j - c]`.
///
/// Rows may differ in length (a missing entry is zero); every output row is `wa + wb - 1` long for the widest
/// rows `wa` of `a` and `wb` of `b`. One `convolve` of the rows laid end to end, `wa + wb - 1` apart,
/// so no row's product spills into the next.
///
/// cost: \(O(S \log S)\) for `S = (a.len + b.len) (wa + wb)`
/// empty: `type Z = Mod<17>; let e: List<List<Z>> = []; convolve_2d([[Z(1)]], e) == [] and convolve_2d(e, [[Z(1)]]) == []`
/// order: rows then columns, lowest degree first; new `List`s, keyed from 0
/// traps: when the product outgrows the largest NTT length `P` supports
/// example: `type Z = Mod<17>; convolve_2d([[Z(1), Z(1)]], [[Z(1)], [Z(2)]]) == [[Z(1), Z(1)], [Z(2), Z(2)]]`
/// example: `type Z = Mod<17>; convolve_2d([[Z(1), Z(1)], [Z(1), Z(1)]], [[Z(1), Z(1)]]) == [[Z(1), Z(2), Z(1)], [Z(1), Z(2), Z(1)]]`
/// example: `type Z = Mod<17>; let e: List<List<Z>> = []; convolve_2d([[Z(1)]], e) == []`
pub fn convolve_2d<P>(a: List<List<Mod<P>>>, b: List<List<Mod<P>>>) -> List<List<Mod<P>>>:
    let wa = max(r.len for r in a) ?? 0
    let wb = max(r.len for r in b) ?? 0
    if wa == 0 or wb == 0: return []
    let stride = wa + wb - 1
    var fa = List.filled(a.len * stride, Mod<P>(0))
    var fb = List.filled(b.len * stride, Mod<P>(0))
    unchecked:
        for (i, r) in a.items:
            for (j, x) in r.items: fa[i * stride + j] = x
        for (i, r) in b.items:
            for (j, x) in r.items: fb[i * stride + j] = x
    let c = convolve(fa, fb)
    [[if i * stride + j < c.len: c[i * stride + j] else: Mod<P>(0) for j in 0..stride] for i in 0..a.len + b.len - 1]

/// The set power series exponential of `a`, length `2^k`, with `a[0] == 0`.
///
/// `b[m]` is the sum, over every partition of the mask `m` into nonempty disjoint blocks, of the
/// product of `a` over the blocks; `b[0] == 1`. With subset convolution as the product, it is
/// `sum_n a^n / n!` and the inverse of `subset_log`.
///
/// cost: \(O(k^2 2^k)\)
/// empty: `let a: List<Mod<17>> = []; subset_exp(a) == []` (traps R0401)
/// order: indexed by mask; a new `List`, keyed from 0
/// traps: when `a.len` is not a power of two, or `a[0] != 0`
/// example: `type Z = Mod<17>; subset_exp([Z(0), Z(1), Z(1), Z(0)]) == [Z(1), Z(1), Z(1), Z(1)]`
/// example: `type Z = Mod<17>; subset_exp([Z(0), Z(1), Z(1), Z(1)]) == [Z(1), Z(1), Z(1), Z(2)]`
pub fn subset_exp<P>(a: List<Mod<P>>) -> List<Mod<P>>:
    assert a.len.is_pow2, "subset_exp needs a power-of-two length"
    assert a[0] == Mod<P>(0), "subset_exp needs `a[0] == 0`"
    ranked(a, true)

/// The set power series logarithm of `a`, length `2^k`, with `a[0] == 1`: the inverse of
/// `subset_exp`, so `subset_exp(subset_log(a)) == a`.
///
/// cost: \(O(k^2 2^k)\)
/// empty: `let a: List<Mod<17>> = []; subset_log(a) == []` (traps R0401)
/// order: indexed by mask; a new `List`, keyed from 0
/// traps: when `a.len` is not a power of two, or `a[0] != 1`
/// example: `type Z = Mod<17>; subset_log([Z(1), Z(1), Z(1), Z(2)]) == [Z(0), Z(1), Z(1), Z(1)]`
/// example: `type Z = Mod<17>; subset_log([Z(1), Z(0)]) == [Z(0), Z(0)]`
pub fn subset_log<P>(a: List<Mod<P>>) -> List<Mod<P>>:
    assert a.len.is_pow2, "subset_log needs a power-of-two length"
    assert a[0] == Mod<P>(1), "subset_log needs `a[0] == 1`"
    ranked(a, false)

fn window<P>(x: List<Mod<P>>, s: Int, w: Int) -> List<Mod<P>>:
    [x[s + t] for t in 0..min(w, x.len - s)]

// Takes the windows rather than the whole lists: a whole `self` list beside `mut self.c` is copied
// at the call.
fn add_block<P>(mut c: List<Mod<P>>, xa: List<Mod<P>>, ya: List<Mod<P>>, base: Int, at: Int):
    if xa.is_empty or ya.is_empty: return
    let prod = times(xa, ya)
    while c.len < base + prod.len: c.push(Mod<P>(0))
    unchecked:
        for t in 0..prod.len:
            if base + t >= at: c[base + t] += prod[t]

fn times<P>(a: List<Mod<P>>, b: List<Mod<P>>) -> List<Mod<P>>:
    if a.is_empty or b.is_empty: return []
    let n = a.len + b.len - 1
    if min(a.len, b.len) <= 32:
        var out = List.filled(n, Mod<P>(0))
        unchecked:
            for i in 0..a.len:
                for j in 0..b.len: out[i + j] += a[i] * b[j]
        return out
    let m = bit_ceil(n)
    var fa = List.filled(m, Mod<P>(0))
    var fb = List.filled(m, Mod<P>(0))
    unchecked:
        for i in 0..a.len: fa[i] = a[i]
        for i in 0..b.len: fb[i] = b[i]
    ntt(mut fa)
    ntt(mut fb)
    unchecked:
        for i in 0..m: fa[i] *= fb[i]
    ntt(mut fa, inverse: true)
    fa.truncate(n)
    fa

fn ranked<P>(a: List<Mod<P>>, exp: Bool) -> List<Mod<P>>:
    let n = a.len
    let k = ctz(n)
    var f = List.filled((k + 1) * n, Mod<P>(0))
    unchecked:
        for m in 0..n: f[popcount(m) * n + m] = a[m]
        for r in 0..=k:
            for bit in 0..k:
                for m in 0..n:
                    if m.bit(bit): f[r * n + m] += f[r * n + (m ^ (1 << bit))]
        var p = List.filled(k + 1, Mod<P>(0))
        var q = List.filled(k + 1, Mod<P>(0))
        for m in 0..n:
            for r in 0..=k: p[r] = f[r * n + m]
            if exp:
                q[0] = Mod<P>(1)
                for t in 1..=k:
                    var acc = Mod<P>(0)
                    for j in 1..=t: acc = acc + Mod<P>(j) * p[j] * q[t - j]
                    q[t] = acc * Mod<P>(t).inv()
            else:
                q[0] = Mod<P>(0)
                for t in 1..=k:
                    var acc = Mod<P>(0)
                    for j in 1..t: acc = acc + Mod<P>(j) * q[j] * p[t - j]
                    q[t] = p[t] - acc * Mod<P>(t).inv()
            for r in 0..=k: f[r * n + m] = q[r]
        for r in 0..=k:
            for bit in 0..k:
                for m in 0..n:
                    if m.bit(bit): f[r * n + m] -= f[r * n + (m ^ (1 << bit))]
    [f[popcount(m) * n + m] for m in 0..n]

@test
fn ntt_matches_the_naive_dft_in_both_orders():
    // Every length up to the largest power of two the prime allows; `Mod<97>` has only
    // `2^5 | P - 1` and a computed primitive root.
    ntt_matches_dft(Mod<NTT_P1>(3), 9)
    ntt_matches_dft(Mod<97>(primitive_root(97)), 5)
    ntt_matches_dft(Mod<17>(primitive_root(17)), 4)

fn ntt_matches_dft<P>(g: Mod<P>, max_lg: Int):
    for lg in 0..=max_lg:
        let n = 1 << lg
        let a = [Mod<P>(rand(0, P - 1)) for _ in 0..n]
        let w = g ** ((P - 1) / n)
        let want = [sum(a[j] * w ** (j * k % n) for j in 0..n) for k in 0..n]
        var f = a.collect()
        ntt(mut f)
        assert f == want
        ntt(mut f, inverse: true)
        assert f == a
        var b = a.collect()
        ntt(mut b, bit_reversed: true)
        for k in 0..n: assert b[k] == want[sum((k >> i & 1) << (lg - 1 - i) for i in 0..lg)]
        ntt(mut b, inverse: true, bit_reversed: true)
        assert b == a

@test
fn ntt_round_trip_and_convolve():
    type Z = Mod<NTT_P1>
    var a = [Z(i * 7 + 3) for i in 0..16]
    let orig = a.collect()
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
fn convolve_matches_brute_force_for_every_value_type():
    for _ in 0..20:
        let (n, m) = (rand(1, 9), rand(1, 9))
        let a = [rand(-50, 50) for _ in 0..n]
        let b = [rand(-50, 50) for _ in 0..m]
        var want = List.filled(n + m - 1, 0)
        for i in 0..n:
            for j in 0..m: want[i + j] += a[i] * b[j]
        assert convolve(a, b) == want
        let az = [Mod<NTT_P1>(x) for x in a]
        let bz = [Mod<NTT_P1>(x) for x in b]
        assert convolve(az, bz) == [Mod<NTT_P1>(x) for x in want]
        let af = [F64(x) for x in a]
        let bf = [F64(x) for x in b]
        let got = convolve(af, bf)
        assert got.len == want.len
        for k in 0..want.len: assert abs(got[k] - F64(want[k])) < 1e-6

@test
fn int_convolve_takes_two_or_three_primes_and_folds_the_cyclic_form():
    for _ in 0..6:
        let (n, m) = (rand(61, 140), rand(61, 140))
        let big = if rand(0, 1) == 0: 50 else: 50_000_000
        let a = [rand(-big, big) for _ in 0..n]
        let b = [rand(-big, big) for _ in 0..m]
        var want = List.filled(n + m - 1, 0)
        for i in 0..n:
            for j in 0..m: want[i + j] += a[i] * b[j]
        assert convolve(a, b) == want
        let c = [rand(-big, big) for _ in 0..n]
        var want_cyc = List.filled(n, 0)
        for i in 0..n:
            for j in 0..n: want_cyc[(i + j) % n] += a[i] * c[j]
        assert convolve(a, c, cyclic: true) == want_cyc
        let z = [Mod<NTT_P1>(x) for x in a]
        let zc = [Mod<NTT_P1>(x) for x in c]
        assert convolve(z, zc, cyclic: true) == [Mod<NTT_P1>(x) for x in want_cyc]

@test
fn sos_and_bitwise_convolutions():
    for bits in 0..=4:
        let n = 1 << bits
        let a = [rand(-20, 20) for _ in 0..n]
        let b = [rand(-20, 20) for _ in 0..n]
        var s = a.collect()
        sos(mut s)
        for mask in 0..n: assert s[mask] == sum(a[sub] for sub in 0..n if (sub & mask) == sub)
        sos(mut s, inverse: true)
        assert s == a
        var u = a.collect()
        sos(mut u, over: Sup)
        for mask in 0..n: assert u[mask] == sum(a[sup] for sup in 0..n if (sup & mask) == mask)
        sos(mut u, over: Sup, inverse: true)
        assert u == a
        var want_or = List.filled(n, 0)
        var want_and = List.filled(n, 0)
        var want_xor = List.filled(n, 0)
        var want_sub = List.filled(n, 0)
        for i in 0..n:
            for j in 0..n:
                want_or[i | j] += a[i] * b[j]
                want_and[i & j] += a[i] * b[j]
                want_xor[i ^ j] += a[i] * b[j]
                if (i & j) == 0: want_sub[i | j] += a[i] * b[j]
        assert convolve(a, b, on: |) == want_or
        assert convolve(a, b, on: &) == want_and
        assert convolve(a, b, on: ^) == want_xor
        assert convolve(a, b, on: subset) == want_sub
        var w = a.collect()
        fwht(mut w)
        fwht(mut w, inverse: true)
        assert w == a
    // Mod<P> works with sos, fwht, and `convolve` on `^` and `subset`
    type Z = Mod<NTT_P1>
    let ma = [Z(1), Z(2), Z(3), Z(4), Z(5), Z(6), Z(7), Z(8)]
    let mb = [Z(9), Z(10), Z(11), Z(12), Z(13), Z(14), Z(15), Z(16)]
    var mw = ma.collect()
    fwht(mut mw)
    fwht(mut mw, inverse: true)
    assert mw == ma
    var want_mxor = List.filled(8, Z(0))
    var want_msub = List.filled(8, Z(0))
    for i in 0..8:
        for j in 0..8:
            want_mxor[i ^ j] += ma[i] * mb[j]
            if (i & j) == 0: want_msub[i | j] += ma[i] * mb[j]
    assert convolve(ma, mb, on: ^) == want_mxor
    assert convolve(ma, mb, on: subset) == want_msub

@test
fn convolve_2d_matches_the_quadruple_loop():
    type Z = Mod<998_244_353>
    let a: List<List<Z>> = [[Z(1), Z(2), Z(3)], [Z(4)], [], [Z(5), Z(6)]]
    let b: List<List<Z>> = [[Z(7), Z(8)], [Z(9), Z(10), Z(11)]]
    let got = convolve_2d(a, b)
    assert got.len == 5 and got.all(r => r.len == 5)
    var want = [[Z(0) for _ in 0..5] for _ in 0..5]
    for (i, r) in a.items:
        for (j, x) in r.items:
            for (k, s) in b.items:
                for (l, y) in s.items: want[i + k][j + l] += x * y
    assert got == want

impl Show for RelaxedConvolve<P>:
    /// The convolution terms returned so far, space-separated.
    ///
    /// example: `type Z = Mod<17>; var rc = RelaxedConvolve<17>(); rc.push(Z(1), Z(3)); rc.push(Z(2), Z(4)); "{rc}" == "3 10"`
    fn show(rc):
        var out = ""
        for i in 0..rc.a.len:
            if not out.is_empty: out += " "
            out += "{rc.c[i]}"
        out

impl Show for SemiRelaxedConvolve<P>:
    /// The convolution terms returned so far, space-separated.
    ///
    /// example: `type Z = Mod<17>; var rc = SemiRelaxedConvolve<17>([Z(1), Z(2)]); rc.push(Z(1)); rc.push(Z(2)); rc.push(Z(5)); "{rc}" == "1 4 9"`
    fn show(rc):
        var out = ""
        for i in 0..rc.a.len:
            if not out.is_empty: out += " "
            out += "{rc.c[i]}"
        out
