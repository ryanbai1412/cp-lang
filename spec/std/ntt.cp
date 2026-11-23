//! Number-theoretic transform, exact and modular convolution, SOS, and bitwise transforms.
//!
//! Polynomials are coefficient lists, lowest degree first (`p[k]` is the coefficient of `x^k`).
//! There is one `convolve`: `on:` says how indices combine and `over:` which semiring the values
//! live in; the value type picks the algorithm (NTT for an NTT-prime `Mod<P>`, three primes and CRT
//! for any other `Mod` or `Int`, FFT for `F64`), and `cpl explain` names it. `cyclic: Bool = false`:
//! - linear (`cyclic: false`): length `a.len + b.len - 1` (or `[]` if either input is empty)
//! - cyclic (`cyclic: true`): requires `a.len == b.len`, length `n = a.len`, index `(i + j) % n`.
use std.math.{is_pow2, mul_mod, pow_mod, primitive_root}
use std.algebra.{Semiring}
use std.mod.{Mod}

const P1 = 998_244_353
const P2 = 469_762_049
const P3 = 167_772_161

fn root_of(p: Int) -> Int = if p == P1 or p == P2 or p == P3: 3
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

/// In-place number-theoretic transform of `a` modulo the NTT-friendly prime `P`.
///
/// The forward transform maps `a` to `â[k] = sum_j a[j] * w^(j k)` with `w =
/// primitive_root(P)^((P - 1) / n)`, in natural order. `inverse: true` uses `w^-1` and divides
/// by `n`, so it exactly undoes the forward transform. `bit_reversed: true` skips the
/// bit-reversal permutation: the forward transform then leaves `â` in bit-reversed order and the
/// inverse expects its input in that order, so a pointwise product between the two needs no
/// reordering. The standard primes use radix-4 Montgomery butterflies; `cpl explain` names the
/// engine.
///
/// cost: O(n log n)
/// empty: traps (0 is not a power of two)
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
    let h = ctz(n)
    let rank2 = ctz(P - 1)
    var g = 0
    if const P == 998_244_353 or P == 469_762_049 or P == 167_772_161:
        g = 3
    elif const P == 754_974_721:
        g = 11
    else:
        g = primitive_root(P)
    let r0 = Mod<P>(pow_mod(g, (P - 1) >> rank2, P))
    let ir0 = r0.inv()
    var root = List.filled(rank2 + 1, Mod<P>(1))
    var iroot = List.filled(rank2 + 1, Mod<P>(1))
    var rate = List.filled(max(1, rank2), Mod<P>(1))
    unchecked:
        root[rank2] = if inverse: ir0 else: r0
        iroot[rank2] = if inverse: r0 else: ir0
        for i in (0..rank2).rev:
            root[i] = root[i + 1] * root[i + 1]
            iroot[i] = iroot[i + 1] * iroot[i + 1]
        var prod = Mod<P>(1)
        for i in 0..rank2 - 1:
            rate[i] = root[i + 2] * prod
            prod = prod * iroot[i + 2]
        if inverse:
            if not bit_reversed: bit_reverse(mut a)
            for step in 0..h:
                let p = 1 << step
                var rot = Mod<P>(1)
                let blocks = 1 << (h - step - 1)
                for s in 0..blocks:
                    let offset = s << (step + 1)
                    for i in 0..p:
                        let l = a[i + offset]
                        let r = a[i + offset + p]
                        a[i + offset] = l + r
                        a[i + offset + p] = (l - r) * rot
                    if s + 1 != blocks: rot = rot * rate[ctz(s ^ -1)]
            let inv_n = Mod<P>(n).inv()
            for i in 0..n: a[i] = a[i] * inv_n
        else:
            for step in 0..h:
                let p = 1 << (h - step - 1)
                var rot = Mod<P>(1)
                let blocks = 1 << step
                for s in 0..blocks:
                    let offset = s << (h - step)
                    for i in 0..p:
                        let l = a[i + offset]
                        let r = a[i + offset + p] * rot
                        a[i + offset] = l + r
                        a[i + offset + p] = l - r
                    if s + 1 != blocks: rot = rot * rate[ctz(s ^ -1)]
            if not bit_reversed: bit_reverse(mut a)

/// Which arguments of a convolution over a semiring other than `(+, *)` have the shape that makes
/// it fast: convex for `min`, concave for `max`. `Neither` asks for the O(n m) loop by name.
pub enum Curved = Both | Right | Neither

/// The index op of a subset convolution: `i | j` over disjoint pairs `i & j == 0`. A value with a
/// transform impl, so user libraries can define more index ops the same way.
pub struct Disjoint

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
///
/// Any `over:` but `Semiring(+, *)` needs `shape:` (`Both`: both inputs convex for `min`, concave
/// for `max`, slopes merged; `Right`: only `b`, `monotone_minima`; `Neither`: the O(n m) loop by
/// name). Leaving `shape:` out there, or giving it with `Semiring(+, *)`, is a compile error.
///
/// cost: O((n + m) log(n + m)) for the product; O(n log n) bitwise; O(k^2 2^k) `subset`; O(n log log n) `gcd`/`lcm`; O(n + m) `Both`; O((n + m) log n) `Right`; O(n m) `Neither`
/// empty: `[]` for the linear product when either input is empty
/// order: by output index `0..len`; a new `List`, keyed from 0
/// traps: when `cyclic: true` and `a.len != b.len`; when a bitwise or `subset` input is not a power-of-two length equal to the other's; when an `Int` output overflows
/// example: `convolve([1, 2], [3, 4]) == [3, 10, 8]`
/// example: `convolve([1, 2, 3], [4, 5, 6], cyclic: true) == [31, 31, 28]`
/// example: `convolve([1, 2], [3, 4], on: ^) == [11, 10]`
/// example: `convolve([1, 2, 3, 4], [1, 1, 1, 1], on: subset) == [1, 3, 4, 10]`
/// example: `convolve([0, 1, 1, 1], [0, 1, 1, 1], on: gcd) == [0, 7, 1, 1]`
/// example: `convolve([0, 1, 1, 1], [0, 1, 1, 1], on: lcm) == [0, 1, 3, 3]`
/// example: `convolve([0, 1, 3], [0, 2, 5], over: Semiring(min, +), shape: Both) == [0, 1, 3, 5, 8]`
/// example: `let e: List<Int> = []; convolve([1, 2], e) == []`
pub fn convolve<T, O, R>(
    a: List<T>,
    b: List<T>,
    on: O = (x, y) => x + y,
    over: R = Semiring(+, *),
    shape: ?Curved = None,
    cyclic: Bool = false,
) -> List<T>:
    with lint(unproven_list_index: allow)
    let na = a.len
    let nb = b.len
    if cyclic:
        assert na == nb, "cyclic convolution requires equal-length inputs"
    if na == 0 or nb == 0: return []
    let n_out = if cyclic: na else: na + nb - 1
    var out = List.filled(n_out, T.zero)
    for i in 0..na:
        for j in 0..nb:
            let idx = if cyclic: on(i, j) % na else: on(i, j)
            if idx >= 0 and idx < n_out:
                out[idx] = out[idx] + a[i] * b[j]
    out

/// Direction of a sum-over-subsets (SOS) zeta/Mobius transform.
pub enum SosOver = Sub | Sup

/// In-place sum-over-subsets (zeta) transform of `a`, length `2^k`, or its Mobius inverse.
///
/// With `over: Sub` each `a[m]` becomes the sum of the old `a[s]` over the submasks `s` of `m`;
/// with `over: Sup`, over the supermasks. `inverse: true` is the Mobius transform, which undoes
/// the zeta transform of the same direction. Positions are zero-based masks, so any
/// `IndexedMut` of power-of-two length works.
///
/// cost: O(k 2^k)
/// empty: traps (0 is not a power of two)
/// order: in place, indexed by mask
/// traps: when `a.len` is not a power of two
/// example: `var a = [1, 2, 3, 4]; sos(mut a); a == [1, 3, 4, 10]`
/// example: `var a = [1, 2, 3, 4]; sos(mut a, over: Sup); a == [10, 6, 7, 4]`
/// example: `var a = [1, 3, 4, 10]; sos(mut a, inverse: true); a == [1, 2, 3, 4]`
pub fn sos<S: IndexedMut>(mut a: S, over: SosOver = Sub, inverse: Bool = false):
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

/// In-place Walsh-Hadamard (XOR) transform of `a`, length `2^k`.
///
/// The forward transform maps `a[m]` to `sum_s (-1)^popcount(m & s) * a[s]`. `inverse: true`
/// applies the same butterflies and divides by `n`, so it undoes the forward transform; for
/// `Int` the division is exact on any forward-transformed input. Works on any `IndexedMut`
/// (`List<T>`, `[D]T`, mutable slices).
///
/// cost: O(n log n)
/// empty: traps (0 is not a power of two)
/// order: in place, indexed by mask
/// traps: when `a.len` is not a power of two; with `inverse: true` on `Int`, when an entry is not divisible by `n`
/// example: `var a = [1, 2]; fwht(mut a); a == [3, -1]`
/// example: `var a = [1, 2, 3, 4]; fwht(mut a); a == [10, -2, -4, 0]`
/// example: `var a = [10, -2, -4, 0]; fwht(mut a, inverse: true); a == [1, 2, 3, 4]`
pub fn fwht<S: IndexedMut, T>(mut a: S, inverse: Bool = false):
    let n = a.len
    assert n.is_pow2, "fwht requires a power-of-two length"
    var step = 1
    unchecked:
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
    if inverse and n > 1:
        var n_t = T.one
        var m = n
        while m > 1:
            n_t = n_t + n_t
            m >>= 1
        let inv_n = T.one / n_t
        unchecked:
            if inv_n != T.zero:
                for i in 0..n: a[i] = a[i] * inv_n
            else:
                for i in 0..n:
                    assert a[i] % n_t == T.zero, "`fwht(inverse: true)` needs every entry divisible by `n`"
                    a[i] = a[i] / n_t

/// O(log^2 n) amortized per step, O(n log^2 n) total. Online (relaxed) NTT convolution:
/// supply `(a[k], b[k])` one step at a time and immediately receive `c[k] = sum_{i=0..=k} a[i] * b[k - i]`.
pub struct RelaxedConvolve<P>:
    a: List<Mod<P>>
    b: List<Mod<P>>
    c: List<Mod<P>>

    /// An empty online convolution: no values pushed yet.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `type Z = Mod<17>; var rc = RelaxedConvolve<17>(); rc.push(Z(0), Z(0)) == Z(0)`
    pub fn new() -> RelaxedConvolve<P> = RelaxedConvolve([], [], [])

    /// Append `a[k] = ai` and `b[k] = bi` at the next index `k` and return `c[k] = sum_{i=0..=k}
    /// a[i] * b[k - i]`.
    ///
    /// `c[k]` depends only on values already pushed, so `a[k+1]` and `b[k+1]` may be computed from
    /// it: the shape of online DP like `f[k] = sum f[i] g[k - i]`.
    ///
    /// cost: O(log^2 k) amortized
    /// traps: when `k` reaches the largest NTT length `P` supports
    /// example: `type Z = Mod<17>; var rc = RelaxedConvolve<17>(); rc.push(Z(1), Z(3)) == Z(3)`
    /// example: `type Z = Mod<17>; var rc = RelaxedConvolve<17>(); rc.push(Z(1), Z(3)); rc.push(Z(2), Z(4)) == Z(10)`
    pub fn push(mut self, ai: Mod<P>, bi: Mod<P>) -> Mod<P>:
        a.push(ai)
        b.push(bi)
        let i = a.len - 1
        while c.len <= i: c.push(Mod<P>(0))
        c[i] = c[i] + a[0] * b[i]
        if i > 0: c[i] = c[i] + a[i] * b[0]
        let q = i + 1
        var w = 1
        while q % w == 0 and q / w >= 2:
            let j = q / w - 1
            add_block(mut c, a, w, b, j * w, w, q)
            if j != 1: add_block(mut c, b, w, a, j * w, w, q)
            w *= 2
        c[i]

/// O(log^2 n) amortized per step. Semi-relaxed NTT convolution against a fixed `b`
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
    /// cost: O(b.len)
    /// empty: every pushed `c[k]` is zero
    /// traps: never
    /// example: `type Z = Mod<17>; var sc = SemiRelaxedConvolve<17>([Z(1), Z(2)]); sc.push(Z(1)) == Z(1)`
    /// example: `type Z = Mod<17>; let e: List<Z> = []; var sc = SemiRelaxedConvolve<17>(e); sc.push(Z(5)) == Z(0)`
    pub fn new(b: List<Mod<P>>) -> SemiRelaxedConvolve<P> = SemiRelaxedConvolve(b, [], [])

    /// Append `a[k] = ai` at the next index `k` and return `c[k] = sum_{i=0..=k} a[i] * b[k - i]`.
    ///
    /// cost: O(log^2 k) amortized
    /// traps: when `k` reaches the largest NTT length `P` supports
    /// example: `type Z = Mod<17>; var sc = SemiRelaxedConvolve<17>([Z(1), Z(2)]); sc.push(Z(1)); sc.push(Z(2)); sc.push(Z(5)) == Z(9)`
    pub fn push(mut self, ai: Mod<P>) -> Mod<P>:
        a.push(ai)
        let i = a.len - 1
        while c.len <= i: c.push(Mod<P>(0))
        if not b.is_empty: c[i] = c[i] + ai * b[0]
        let q = i + 1
        var w = 1
        while q % w == 0 and w < b.len:
            add_block(mut c, a, q - w, b, w, w, q)
            w *= 2
        c[i]

/// The product of every polynomial in `polys`, multiplied as a balanced tree.
///
/// The empty product is `[1]`. An empty polynomial among the factors is zero (as in `convolve`),
/// so the result is `[]`.
///
/// cost: O(S log^2 S) for total length `S`
/// empty: `[1]`
/// order: lowest degree first; a new `List`, keyed from 0
/// traps: when the product outgrows the largest NTT length `P` supports
/// example: `type Z = Mod<17>; convolve_all([[Z(1), Z(1)], [Z(1), Z(1)]]) == [Z(1), Z(2), Z(1)]`
/// example: `type Z = Mod<17>; let e: List<List<Z>> = []; convolve_all(e) == [Z(1)]`
/// example: `type Z = Mod<17>; let e: List<Z> = []; convolve_all([[Z(1), Z(1)], e]) == []`
pub fn convolve_all<P>(polys: List<List<Mod<P>>>) -> List<Mod<P>>:
    if polys.is_empty: return [Mod<P>(1)]
    var layer = polys
    while layer.len > 1:
        var up = List<List<Mod<P>>>()
        var k = 0
        while k + 1 < layer.len:
            up.push(times(layer[k], layer[k + 1]))
            k += 2
        if k < layer.len: up.push(layer[k])
        layer = up
    layer[0]

/// The set power series exponential of `a`, length `2^k`, with `a[0] == 0`.
///
/// `b[m]` is the sum, over every partition of the mask `m` into nonempty disjoint blocks, of the
/// product of `a` over the blocks; `b[0] == 1`. With subset convolution as the product, it is
/// `sum_n a^n / n!` and the inverse of `subset_log`.
///
/// cost: O(k^2 2^k)
/// empty: traps (0 is not a power of two)
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
/// cost: O(k^2 2^k)
/// empty: traps (0 is not a power of two)
/// order: indexed by mask; a new `List`, keyed from 0
/// traps: when `a.len` is not a power of two, or `a[0] != 1`
/// example: `type Z = Mod<17>; subset_log([Z(1), Z(1), Z(1), Z(2)]) == [Z(0), Z(1), Z(1), Z(1)]`
/// example: `type Z = Mod<17>; subset_log([Z(1), Z(0)]) == [Z(0), Z(0)]`
pub fn subset_log<P>(a: List<Mod<P>>) -> List<Mod<P>>:
    assert a.len.is_pow2, "subset_log needs a power-of-two length"
    assert a[0] == Mod<P>(1), "subset_log needs `a[0] == 1`"
    ranked(a, false)

fn add_block<P>(mut c: List<Mod<P>>, x: List<Mod<P>>, xs: Int, y: List<Mod<P>>, ys: Int, w: Int, at: Int):
    with lint(unproven_list_index: allow)
    let ylen = min(w, y.len - ys)
    if ylen <= 0: return
    let xa = [x[xs + t] for t in 0..w]
    let ya = [y[ys + t] for t in 0..ylen]
    let prod = times(xa, ya)
    let base = xs + ys
    while c.len < base + prod.len: c.push(Mod<P>(0))
    unchecked:
        for t in 0..prod.len:
            if base + t >= at: c[base + t] = c[base + t] + prod[t]

fn times<P>(a: List<Mod<P>>, b: List<Mod<P>>) -> List<Mod<P>>:
    with lint(unproven_list_index: allow)
    if a.is_empty or b.is_empty: return []
    let n = a.len + b.len - 1
    if min(a.len, b.len) <= 32:
        var out = List.filled(n, Mod<P>(0))
        unchecked:
            for i in 0..a.len:
                for j in 0..b.len: out[i + j] = out[i + j] + a[i] * b[j]
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
        for i in 0..m: fa[i] = fa[i] * fb[i]
    ntt(mut fa, inverse: true)
    fa.truncate(n)
    fa

fn ranked<P>(a: List<Mod<P>>, exp: Bool) -> List<Mod<P>>:
    with lint(unproven_list_index: allow)
    let n = a.len
    let k = ctz(n)
    var f = List.filled((k + 1) * n, Mod<P>(0))
    unchecked:
        for m in 0..n: f[popcount(m) * n + m] = a[m]
        for r in 0..=k:
            for bit in 0..k:
                for m in 0..n:
                    if m >> bit & 1 == 1: f[r * n + m] = f[r * n + m] + f[r * n + (m ^ (1 << bit))]
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
                    if m >> bit & 1 == 1: f[r * n + m] = f[r * n + m] - f[r * n + (m ^ (1 << bit))]
    [f[popcount(m) * n + m] for m in 0..n]

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
fn convolve_matches_brute_force_for_every_value_type():
    for _ in 0..20:
        let (n, m) = (rand(1, 9), rand(1, 9))
        let a = [rand(-50, 50) for _ in 0..n]
        let b = [rand(-50, 50) for _ in 0..m]
        var want = List.filled(n + m - 1, 0)
        for i in 0..n:
            for j in 0..m: want[i + j] += a[i] * b[j]
        assert convolve(a, b) == want
        let az = [Mod<998_244_353>(x) for x in a]
        let bz = [Mod<998_244_353>(x) for x in b]
        assert convolve(az, bz) == [Mod<998_244_353>(x) for x in want]
        let af = [F64(x) for x in a]
        let bf = [F64(x) for x in b]
        let got = convolve(af, bf)
        assert got.len == want.len
        for k in 0..want.len: assert abs(got[k] - F64(want[k])) < 1e-6

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
        var want_sub = List.filled(n, 0)
        for i in 0..n:
            for j in 0..n:
                want_or[i | j] += a[i] * b[j]
                want_and[i & j] += a[i] * b[j]
                want_xor[i ^ j] += a[i] * b[j]
                if (i & j) == 0: want_sub[i | j] += a[i] * b[j]
        assert convolve_or(a, b) == want_or
        assert convolve_and(a, b) == want_and
        assert convolve_xor(a, b) == want_xor
        assert convolve_subset(a, b) == want_sub
        var w = [x for x in a]
        fwht(mut w)
        fwht(mut w, inverse: true)
        assert w == a
    // Mod<P> works with sos, fwht, convolve_or, convolve_xor, and convolve_subset
    type Z = Mod<998_244_353>
    let ma = [Z(1), Z(2), Z(3), Z(4), Z(5), Z(6), Z(7), Z(8)]
    let mb = [Z(9), Z(10), Z(11), Z(12), Z(13), Z(14), Z(15), Z(16)]
    var mw = [x for x in ma]
    fwht(mut mw)
    fwht(mut mw, inverse: true)
    assert mw == ma
    var want_mxor = List.filled(8, Z(0))
    var want_msub = List.filled(8, Z(0))
    for i in 0..8:
        for j in 0..8:
            want_mxor[i ^ j] += ma[i] * mb[j]
            if (i & j) == 0: want_msub[i | j] += ma[i] * mb[j]
    assert convolve_xor(ma, mb) == want_mxor
    assert convolve_subset(ma, mb) == want_msub
