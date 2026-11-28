//! Modular arithmetic and counting.
//!
//! start: `Mod`, `Comb`, `Mod.inv`, `Barrett`
//! example: `Mod<7>(3) * Mod<7>(5) == Mod<7>(1) and Mod<7>(3).inv() == Mod<7>(5)`
//!
//! `Mod<P>` is a number modulo a constant `P`: the modulus is part of the
//! type, takes no memory, and two moduli cannot be mixed by accident.
//! The modulus may be a constant (`type Z = Mod<998244353>`) or a value read from
//! the input (`let p: Int = read()`, then `type Z = Mod(p)`): one type serves both.
//! `P` may be anything below 2^62 (so `a + b` fits): above 3.03e9, where `P * P` no longer fits
//! an `Int`, `*` multiplies in 128 bits.
use std.algebra.{Monoid}
use std.fenwick.{Fenwick}
use std.math.{crt_all, factor, gcd, inv_mod, isqrt, mod_log, mod_sqrt, mul_mod, pow_mod}
use std.range.{Len}
use std.list.{List, Str}
use std.ops.{Add, Mul, Sub, min, max, Div, Neg, One, Pow, Zero}
use std.optional.{Some}
use std.read.{Read, read}
use std.show.{Show}

/// An integer modulo the constant `P`, always stored in `0..P`.
/// `+ - *` are \(O(1)\); `/` and `inv` are \(O(\log P)\) and need `gcd(x, P) == 1`.
pub struct Mod<P>(pub v: 0..P):
    /// The modulus: `Z.MOD`.
    const MOD = P

    /// Any `Int`, reduced into `0..P` by Euclidean remainder: `Mod<7>(-1)` is 6.
    ///
    /// `Mod<P>(x)` calls `new`, so every way of building a `Mod` reduces.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Mod<7>(-1) == Mod<7>(6)`
    /// example: `Mod<7>(15) == Mod<7>(1)`
    /// example: `Mod<1>(5) == Mod<1>(0)`
    pub fn new(x: Int) -> Mod<P> = Mod(x % P)

    /// The multiplicative inverse when `gcd(v, P) == 1`, or `None` otherwise.
    ///
    /// Uses the extended Euclidean algorithm, so `P` need not be prime. With `P == 1` the only value
    /// is 0, which is also `one`, so `Mod<1>(0).try_inv()` is `Some(Mod<1>(0))`.
    ///
    /// cost: \(O(\log P)\)
    /// traps: never
    /// example: `Mod<7>(3).try_inv() == Some(Mod<7>(5))`
    /// example: `Mod<6>(4).try_inv() == None`
    /// example: `Mod<7>(0).try_inv() == None`
    pub fn try_inv(self) -> ?Mod<P>:
        var (a, b, x, y) = (v, P, 1, 0)
        while b != 0:
            let q = a / b
            (a, b) = (b, a - q * b)
            (x, y) = (y, x - q * y)
        if a != 1: return None
        Mod(x % P)

    /// The multiplicative inverse, by the extended Euclidean algorithm (so `P` need not be prime).
    ///
    /// `x.inv()` is `x.try_inv().unwrap()` with a trap that names `x` and `P`; `a / b` is
    /// `a * b.inv()`.
    ///
    /// cost: \(O(\log P)\)
    /// traps: when `gcd(v, P) != 1`
    /// example: `Mod<7>(3).inv() == Mod<7>(5)`
    /// example: `Mod<10>(3).inv() == Mod<10>(7)`
    @lint(fermat_inverse, from: "pow_mod(a, P - 2, P)", to: "Mod<P>(a).inv().v", holes: "a, P")
    pub fn inv(self) -> Mod<P> = self.try_inv()!

    /// Lift an `Int` to `Mod<P>`; the same reduction as `new`.
    ///
    /// This is the conversion an `Int` literal takes where a `Mod<P>` is expected, so
    /// `let x: Mod<7> = 10` is `Mod<7>.from_int(10)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Mod<7>.from_int(10) == Mod<7>(3)`
    /// example: `Mod<7>.from_int(-8) == Mod<7>(6)`
    pub fn from_int(x: Int) -> Mod<P> = Mod(x % P)

    /// A square root modulo the prime `P` (Tonelli-Shanks), or `None` when there is None.
    ///
    /// Of the two roots `r` and `P - r`, it returns the one with the smaller value `v`; the root of 0
    /// is 0.
    ///
    /// cost: \(O(\log^2 P)\)
    /// ties: `Mod<11>(5).sqrt() == Some(Mod<11>(4))`
    /// traps: when `P` is not prime
    /// example: `Mod<7>(2).sqrt() == Some(Mod<7>(3))`
    /// example: `Mod<7>(3).sqrt() == None`
    /// example: `Mod<7>(0).sqrt() == Some(Mod<7>(0))`
    pub fn sqrt(self) -> ?Mod<P>:
        let r = mod_sqrt(v, P) else: return None
        Some(Mod(min(r, (P - r) % P)))

    /// The least `k >= 0` with `self ** k == target` (baby-step giant-step), or `None`.
    ///
    /// Any modulus works: the non-coprime part is peeled off first. `self ** 0` is `one`, so
    /// `x.log(one)` is `Some(0)` for every `x`.
    ///
    /// cost: \(O(\sqrt P)\)
    /// ties: `Mod<7>(2).log(Mod<7>(2)) == Some(1)`
    /// traps: never
    /// example: `Mod<7>(3).log(Mod<7>(6)) == Some(3)`
    /// example: `Mod<7>(3).log(Mod<7>(1)) == Some(0)`
    /// example: `Mod<7>(2).log(Mod<7>(3)) == None`
    pub fn log(self, target: Mod<P>) -> ?Int = mod_log(v, target.v, P)

    /// The inverse of every element of `xs`, with one `inv` in all (Montgomery's trick).
    ///
    /// cost: \(O(n + \log P)\)
    /// empty: `Mod<7>.inv_all([]) == []`
    /// order: same order as `xs`; a new `List`, keyed from 0
    /// traps: when any element has `gcd(v, P) != 1` (in particular 0)
    /// example: `Mod<7>.inv_all([Mod<7>(1), Mod<7>(2), Mod<7>(3)]) == [Mod<7>(1), Mod<7>(4), Mod<7>(5)]`
    /// example: `let e: List<Mod<7>> = []; Mod<7>.inv_all(e) == []`
    pub fn inv_all(xs: List<Mod<P>>) -> List<Mod<P>>:
        var pre: List<Mod<P>> = [Mod(1 % P)]
        for x in xs: pre.push(pre.last! * x)
        var out = List.filled(xs.len, Mod(0))
        var acc = pre[xs.len].inv()
        var i = xs.len
        while i > 0:
            i -= 1
            out[i] = acc * pre[i]
            acc *= xs[i]
        out

impl Zero for Mod<P>:
    @get fn zero() -> Mod<P> = Mod(0)

impl One for Mod<P>:
    @get fn one() -> Mod<P> = Mod(1 % P)

impl Add for Mod<P>:
    // Reduce first and test the sign: one sum and a select, where comparing the sum
    // against `P` recomputes it on both arms.
    fn add(a, b):
        let r = a.v + b.v - P
        Mod(if r < 0: r + P else: r)

impl Sub for Mod<P>:
    fn sub(a, b) = Mod(if a.v >= b.v: a.v - b.v else: a.v - b.v + P)

impl Neg for Mod<P>:
    fn neg(a) = Mod(if a.v == 0: 0 else: P - a.v)

impl Mul for Mod<P>:
    // The product of two residues fits an `Int` only below 3_037_000_500; a larger modulus
    // multiplies in 128 bits. With a constant `P` the test is settled when the C++ is built.
    fn mul(a, b) = if P < 3_037_000_500: Mod(a.v * b.v % P) else: Mod(mul_mod(a.v, b.v, P))

impl Div for Mod<P>:
    fn div(a, b) = a * b.inv()

impl Pow for Mod<P>:
    fn pow(a, e: Int) -> Mod<P>:
        if e < 0: return a.inv() ** (-e)
        Mod(pow_mod(a.v, e, P))

/// A number held as its residue in `0..modulus()`, so a sum of many residue products can stay
/// unreduced and be reduced once: `Mat` products use it. `Mod` has one; a user modular type joins
/// by implementing it the same way.
pub trait Residue:
    /// The modulus: every residue is in `0..modulus()`.
    fn modulus() -> Int

    /// This value's residue, in `0..modulus()`.
    fn residue(self) -> Int

    /// The value congruent to `x`, for `x >= 0`.
    fn reduce(x: Int) -> Self

impl Residue for Mod<P>:
    fn modulus() -> Int = P
    fn residue(self) -> Int = self.v
    fn reduce(x: Int) -> Mod<P> = Mod(x % P)

impl Show for Mod<P>:
    fn show(a) = "{a.v}"

/// One signed integer token of any length, reduced modulo `P`, so a
/// 10^4-digit coefficient is `read<Mod<P>>()`. (R10.2.3)
///
/// trap: `read<Mod<998244353>>()` (stdin `1x`)
impl Read for Mod<P>:
    // Old-pipeline compat: the frozen old `cpl` reads this file and has no `Input`, so this impl
    // keeps `fn read()` and reads standard input even under `read(from:)`. It takes
    // `mut input: Input` in stage 5's switching landing (handbook/design/next/migration.md).
    fn read() -> Mod<P>:
        let s: Str = read()
        // `Mod` arithmetic, so `r * 10` cannot overflow for a modulus near 2^61.
        let ten = Mod(10 % P)
        var r = Mod(0)
        for c in s:
            if c != '-':
                assert '0' <= c <= '9', "`read<Mod<P>>()` reads an integer token"
                r = r * ten + Mod((Int(c) - Int('0')) % P)
        return if s[0] == '-': -r else: r

/// Factorials and inverse factorials over any `Residue` type `M` with a prime modulus, such as `Mod<P>`.
/// Its tables are `@cache` fields that grow on demand, so an immutable
/// `let c = Comb<Z>()` (or `Comb<Z>(limit)` to pre-size) answers queries in \(O(1)\)
/// with amortized \(O(1)\) growth per new factorial.
pub struct Comb<M: Residue + Add + Sub + Mul>:
    @cache f: List<M>
    @cache g: List<M>
    @cache d: List<M>
    @cache iv: List<M>

    /// Precompute factorials and inverse factorials up to `limit` for `M = Mod<P>`, `P` prime.
    ///
    /// The tables hold `min(limit, P - 1) + 1` entries and grow on demand later (doubling, capped at
    /// `P`), so `Comb<Z>()` with no limit is valid and simply grows on first use.
    ///
    /// cost: \(O(\min(\mathit{limit}, P))\)
    /// traps: when `limit < 0` or `limit >= 2^31`, or `P` is not prime
    /// example: `type Z = Mod<7>; let c = Comb<Z>(3); c.fact(3) == Z(6)`
    pub fn new(limit: Len = 0) -> Comb<M>:
        assert limit >= 0, "`Comb(limit)` needs `limit >= 0`"
        let p = M.modulus()
        for q in 2..=p.isqrt:
            assert p % q != 0, "`Comb` needs a prime modulus"
        let c = Comb<M>([M.reduce(1)], [M.reduce(1)], [M.reduce(1), M.reduce(0)], [])
        c.grow(limit)
        c

    fn grow(self, want: Int):
        let have = f.len
        let n = min(max(want + 1, 2 * have), M.modulus())
        if n <= have: return
        f.reserve(n)
        g.resize(n, M.reduce(1))
        // Each step carries the running product in a local, so the serial chain of
        // multiplies never waits on a store of the element just written.
        var acc = f[have - 1]
        for i in have..n:
            acc *= M.reduce(i)
            f.push(acc)
        var inv = M.reduce(inv_mod(acc.residue(), M.modulus())!)
        g[n - 1] = inv
        for i in (have..n - 1).rev:
            inv *= M.reduce(i + 1)
            g[i] = inv

    fn ensure(self, n: Int):
        if n >= f.len: self.grow(n)

    /// `n!`, reduced; 0 when `n >= M.modulus()` (the product contains `P`).
    ///
    /// cost: \(O(1)\) amortized
    /// traps: when `n < 0`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>().fact(5) == Z(120)`
    /// example: `type Z = Mod<7>; Comb<Z>().fact(6) == Z(6)`
    /// example: `type Z = Mod<7>; Comb<Z>().fact(7) == Z(0)`
    pub fn fact(self, n: 0..) -> M:
        assert n >= 0, "`fact(n)` needs `n >= 0`"
        if n >= M.modulus(): return M.reduce(0)
        self.ensure(n)
        f[n]

    /// `1 / n!`, for `0 <= n < M.modulus()`.
    ///
    /// cost: \(O(1)\) amortized
    /// traps: when `n` is outside `0..M.modulus()`
    /// example: `type Z = Mod<7>; Comb<Z>().inv_fact(3) == Z(6)`
    /// example: `type Z = Mod<7>; Comb<Z>().inv_fact(0) == Z(1)`
    pub fn inv_fact(self, n: Int) -> M:
        assert 0 <= n < M.modulus(), "`inv_fact(n)` needs `n` in `0..M.modulus()`"
        self.ensure(n)
        g[n]

    /// `1 / i`, for `1 <= i < M.modulus()`: one load from a table of `(i - 1)! / i!`, built from the
    /// factorials on the first call and kept as long as they are.
    ///
    /// cost: \(O(1)\) amortized
    /// traps: when `i` is outside `1..M.modulus()`
    /// example: `type Z = Mod<7>; Comb<Z>().inv(3) == Z(5)`
    pub fn inv(self, i: Int) -> M:
        assert 1 <= i < M.modulus(), "`inv(i)` needs `i` in `1..M.modulus()`"
        if i >= iv.len: self.grow_inv(i)
        iv[i]

    // `iv[k] = (k - 1)! / k!` for every `k < f.len`: one load per `inv`, and a Comb that never
    // calls `inv` keeps no third table.
    fn grow_inv(self, i: Int):
        self.ensure(i)
        let have = iv.len
        iv.reserve(f.len)
        for k in have..f.len: iv.push(if k == 0: M.reduce(0) else: g[k] * f[k - 1])

    fn choose_lucas(self, n: Int, k: Int) -> M:
        let p = M.modulus()
        self.choose(n / p, k / p) * self.choose(n % p, k % p)

    /// The number of `k`-subsets of `n` things, `n! / (k! (n - k)!)`; 0 when `k` is outside `0..=n`.
    ///
    /// For `n < M.modulus()` it reads the tables. For `n >= M.modulus()` it uses Lucas's theorem on the base-`P`
    /// digits, so the answer is exact modulo `P` for every `n`. A negative `n` gives 0.
    ///
    /// cost: \(O(1)\) amortized for `n < M.modulus()`; \(O(\mathit{log\_P} n)\) otherwise
    /// traps: never
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>().choose(5, 2) == Z(10)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>().choose(5, 6) == Z(0)`
    /// example: `type Z = Mod<7>; Comb<Z>().choose(10, 3) == Z(1)`
    /// example: `type Z = Mod<7>; Comb<Z>().choose(7, 1) == Z(0)`
    pub fn choose(self, n: Int, k: Int) -> M:
        if n < 0 or k < 0 or k > n: return M.reduce(0)
        if n >= M.modulus(): return self.choose_lucas(n, k)
        self.ensure(n)
        f[n] * g[k] * g[n - k]

    /// The number of ordered selections of `k` out of `n` things, `n! / (n - k)!`; 0 when `k` is
    /// outside `0..=n`.
    ///
    /// The value is the falling factorial `n (n - 1) ... (n - k + 1)` reduced modulo `P`: 0 when
    /// those `k` numbers include a multiple of `P`, else `fact(n % P) / fact((n - k) % P)`, so it is
    /// exact for `n >= M.modulus()` too.
    ///
    /// cost: \(O(1)\) amortized
    /// traps: never
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>().arrange(5, 2) == Z(20)`
    /// example: `type Z = Mod<7>; Comb<Z>().arrange(9, 2) == Z(2)`
    /// example: `type Z = Mod<7>; Comb<Z>().arrange(8, 2) == Z(0)`
    pub fn arrange(self, n: Int, k: Int) -> M:
        if n < 0 or k < 0 or k > n: return M.reduce(0)
        let p = M.modulus()
        if n / p != (n - k) / p: return M.reduce(0)
        self.fact(n % p) * self.inv_fact((n - k) % p)

    /// The number of ways to put `n` equal balls into `k` boxes (stars and bars): `choose(n + k - 1, n)`.
    ///
    /// `multichoose(0, 0)` is 1 (the empty placement) and `multichoose(n, 0)` is 0 for `n > 0`; a
    /// negative `n` or `k` gives 0.
    ///
    /// cost: \(O(1)\) amortized
    /// traps: never
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>().multichoose(3, 2) == Z(4)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>().multichoose(2, 3) == Z(6)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>().multichoose(0, 0) == Z(1)`
    pub fn multichoose(self, n: Int, k: Int) -> M:
        if n == 0: return M.reduce(1)
        if k <= 0 or n < 0: return M.reduce(0)
        self.choose(n + k - 1, n)

    /// The `n`-th Catalan number, `choose(2n, n) - choose(2n, n + 1)`.
    ///
    /// The difference form needs no division by `n + 1`, so it stays exact when `n + 1` is a
    /// multiple of `P`.
    ///
    /// cost: \(O(1)\) amortized
    /// traps: when `n < 0`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>().catalan(0) == Z(1)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>().catalan(4) == Z(14)`
    /// example: `type Z = Mod<7>; Comb<Z>().catalan(4) == Z(0)`
    pub fn catalan(self, n: 0..) -> M:
        assert n >= 0, "`catalan(n)` needs `n >= 0`"
        self.choose(2 * n, n) - self.choose(2 * n, n + 1)

    /// The number of derangements `!n`: permutations of `n` things with no fixed point.
    ///
    /// `!0 = 1`, `!1 = 0`, and `!n = (n - 1)(!(n - 1) + !(n - 2))`; the values are tabled like the
    /// factorials.
    ///
    /// cost: \(O(1)\) amortized, \(O(n)\) table
    /// traps: when `n < 0`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>().derange(0) == Z(1)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>().derange(4) == Z(9)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>().derange(5) == Z(44)`
    pub fn derange(self, n: 0..) -> M:
        assert n >= 0, "`derange(n)` needs `n >= 0`"
        while d.len <= n:
            let i = d.len
            d.push(M.reduce(i - 1) * (d[i - 1] + d[i - 2]))
        d[n]

    /// The multinomial coefficient `(sum ks)! / prod(k!)`; 0 if any `k < 0`.
    ///
    /// It is computed as the product of `choose(k_1 + ... + k_i, k_i)`, so Lucas applies and the
    /// result is exact modulo `P` even when `sum ks >= M.modulus()`.
    ///
    /// cost: \(O(\mathit{ks}.\mathit{len})\) amortized
    /// empty: `type Z = Mod<1_000_000_007>; Comb<Z>().multinomial([]) == Z(1)`
    /// traps: never
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>().multinomial([2, 1]) == Z(3)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>().multinomial([1, 1, 1]) == Z(6)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>().multinomial([2, -1]) == Z(0)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>().multinomial([]) == Z(1)`
    pub fn multinomial(self, ks: List<Int>) -> M:
        var acc = M.reduce(1)
        var total = 0
        for k in ks:
            if k < 0: return M.reduce(0)
            total += k
            acc = acc * self.choose(total, k)
        acc

/// Fast Barrett modular multiplier for a runtime modulus `1 <= m < 2^31`. \(O(1)\) reduction.
pub struct Barrett(pub m: Int, im: U64):
    /// A Barrett multiplier for the runtime modulus `m`, `1 <= m < 2^31`: precomputes
    /// `im = floor((2^64 - 1) / m) + 1` (wrapping, so 0 for `m == 1`).
    ///
    /// cost: \(O(1)\)
    /// traps: when `m` is outside `1..2^31`
    /// example: `Barrett(7).mul(3, 5) == 1`
    pub fn new(m: Int) -> Barrett:
        assert 1 <= m < 2_147_483_648
        Barrett(m, if m == 1: U64(0) else: (U64(0) - U64(1)) / U64(m) + U64(1))

    /// `(a * b) % m` without a hardware division, for `a, b` in `0..m`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `a` or `b` is outside `0..m`
    /// example: `Barrett(7).mul(3, 5) == 1`
    /// example: `Barrett(1_000_000_007).mul(1_000_000_006, 1_000_000_006) == 1`
    /// example: `Barrett(1).mul(0, 0) == 0`
    pub fn mul(self, a: Int, b: Int) -> Int:
        assert 0 <= a and a < m and 0 <= b and b < m
        self.reduce(U64(a) * U64(b))

    /// `a % m`, for `a < m * m`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `a >= m * m`
    /// example: `Barrett(7).reduce(48) == 6`
    /// example: `Barrett(7).reduce(0) == 0`
    pub fn reduce(self, a: U64) -> Int:
        assert a < U64(m) * U64(m)
        let q = Int((I128(a) * I128(im)) >> 64)
        let r = Int(a) - q * m
        if r < 0: r + m else: r

/// \(O(p^e)\) build, \(O(\mathit{log\_p} n)\) query. Binomial coefficients modulo a prime power `p^e` (Granville's theorem).
pub struct PrimePowerComb(p: Int, e: Int, pe: Int):
    free: List<Int>

    /// Tables for binomial coefficients modulo `p^e`: the factorials with every factor of `p`
    /// removed, modulo `p^e`.
    ///
    /// cost: \(O(p^e)\) time and memory
    /// traps: when `p` is not prime, or `e < 1`, or `p^e` overflows `Int`
    /// example: `PrimePowerComb(2, 3).choose(6, 3) == 4`
    pub fn new(p: Int, e: Int) -> PrimePowerComb:
        assert p >= 2 and e >= 1
        assert factor(p) == [(p, 1)], "`PrimePowerComb(p, e)` needs a prime `p`"
        var pe = 1
        for _ in 0..e:
            assert pe <= Int.MAX / p, "`p^e` overflows `Int`"
            pe *= p
        var free = List.filled(pe + 1, 1)
        for i in 1..=pe: free[i] = if i % p == 0: free[i - 1] else: free[i - 1] * (i % pe) % pe
        PrimePowerComb(p, e, pe, free)

    fn free_fact(self, n: Int) -> Int:
        var acc = 1
        var m = n
        while m > 0:
            if (m / pe) % 2 == 1: acc = acc * free[pe] % pe
            acc = acc * free[m % pe] % pe
            m /= p
        acc

    fn p_count(self, n: Int) -> Int:
        var c = 0
        var m = n / p
        while m > 0:
            c += m
            m /= p
        c

    /// `C(n, k) mod p^e`, by Granville's generalization of Lucas's theorem; 0 when `k` is outside
    /// `0..=n`.
    ///
    /// cost: \(O(\mathit{log\_p} n)\)
    /// traps: never
    /// example: `PrimePowerComb(2, 3).choose(6, 3) == 4`
    /// example: `PrimePowerComb(3, 2).choose(9, 3) == 3`
    /// example: `PrimePowerComb(3, 2).choose(5, 7) == 0`
    pub fn choose(self, n: Int, k: Int) -> Int:
        if k < 0 or k > n: return 0
        let c = self.p_count(n) - self.p_count(k) - self.p_count(n - k)
        if c >= e: return 0
        var r = self.free_fact(n) * inv_mod(
            self.free_fact(k) * self.free_fact(n - k) % pe,
            pe,
        )! % pe
        for _ in 0..c: r = r * p % pe
        r

/// \(O(\text{sum p\_i}^\mathit{e\_i})\) build, \(O(\mathit{omega}(m) \log n)\) query. Binomial coefficients modulo any `m >= 1` via CRT.
pub struct ModComb(pub m: Int):
    parts: List<PrimePowerComb>

    /// Binomial coefficients modulo any `m >= 1`: factor `m` and build one `PrimePowerComb` per
    /// prime power, combined by CRT.
    ///
    /// cost: \(O(\sum_i p_i^{e_i})\) for \(m = \prod_i p_i^{e_i}\)
    /// traps: when `m < 1`
    /// example: `ModComb(12).choose(6, 3) == 8`
    pub fn new(m: 1..) -> ModComb:
        assert m >= 1, "`ModComb(m)` needs `m >= 1`"
        ModComb(m, [PrimePowerComb.new(q, k) for (q, k) in factor(m)])

    /// `C(n, k) mod m`, by CRT over the prime powers of `m`; 0 when `k` is outside `0..=n`.
    ///
    /// With `m == 1` every answer is 0.
    ///
    /// cost: \(O(\mathit{omega}(m) \log n)\)
    /// traps: never
    /// example: `ModComb(12).choose(6, 3) == 8`
    /// example: `ModComb(10).choose(5, 2) == 0`
    /// example: `ModComb(1).choose(5, 2) == 0`
    pub fn choose(self, n: Int, k: Int) -> Int:
        if m == 1 or k < 0 or k > n: return 0
        crt_all([(q.choose(n, k), q.pe) for q in parts])!.0

@test
fn values_stay_reduced():
    type M = Mod<7>
    assert M(-1).v == 6 and M(15).v == 1 and M(7) == M(0)
    assert M(3) + M(5) == M(1)
    assert M(3) - M(5) == M(5)
    assert -M(3) == M(4) and -M(0) == M(0)
    assert M(3) * M(5) == M(1)
    assert M(1) / M(3) == M(5)
    assert "{M(10)}" == "3"

@test
fn pow_and_inverse():
    type M = Mod<998244353>
    for x in [1, 2, 3, 12345, 998244352]:
        assert M(x) * M(x).inv() == M.one
        assert M(x) ** 998244352 == M.one
        assert M(x) ** (-1) == M(x).inv()
        var p = M.one
        for e in 0..20:
            assert M(x) ** e == p
            p = p * M(x)
    // a composite modulus still has inverses for coprime values
    assert Mod<10>(3).inv() == Mod<10>(7)

@test
fn mod_values_work_in_generic_structures():
    type M = Mod<1000000007>
    let xs = [M(0), M(1), M(5), M(-3), M(123456789)]
    assert laws_hold(Monoid<M>.of(+), xs)
    assert laws_hold(Monoid<M>.of(*), xs)
    domain V = 0..5
    var f = Fenwick<V, M>()
    for v in V: f.add(v, M(10) ** (9 + v.ord))
    assert f.fold(V(0)..=V(4)) == M(1000000000) * M(11111)

@test
fn choose_matches_pascals_triangle():
    type M = Mod<1000000007>
    // Immutable binding: Comb<M> auto-grows via interior mutability
    let c = Comb<M>()
    var row = [M(1)]
    for n in 0..=30:
        for k in 0..=n: assert c.choose(n, k) == row[k]
        assert c.choose(n, n + 1) == M(0) and c.choose(n, -1) == M(0)
        row = [M(1)] + [row[k] + row[k + 1] for k in 0..n] + [M(1)]
    assert c.fact(5) == M(120) and c.fact(20) * c.inv_fact(20) == M(1)
    assert c.arrange(5, 2) == M(20)
    assert c.multichoose(3, 2) == M(4)
    assert [c.catalan(n).v for n in 0..8] == [1, 1, 2, 5, 14, 42, 132, 429]
    // Auto-growth on a small prime clamps before P so (P-1)! stays invertible
    let small = Comb<Mod<11>>()
    assert small.choose(10, 4) == Mod<11>(210)
    assert small.fact(10) == Mod<11>(10)
    // Pre-sizing allocates exact capacity `limit + 1`
    let pre = Comb<M>(40)
    assert pre.choose(40, 20) == c.choose(40, 20)

@test
fn try_inv_and_modulus_one():
    assert Mod<10>(3).try_inv() == Mod<10>(7)
    assert Mod<10>(2).try_inv() == None
    assert Mod<10>(0).try_inv() == None
    assert Mod<1>(0) == Mod<1>(42) and Mod<1>(0).try_inv() == Mod<1>(0)
    assert Mod<1>.zero == Mod<1>.one

@test
fn a_modulus_read_at_run_time():
    for p in [2, 12, 1_000_000_007]:
        type Z = Mod(p)
        assert Z(p) == Z(0) and Z(-1) + Z(1) == Z(0)
        assert Z(5) ** 3 == Z(125 % p) and Z(7) ** 0 == Z(1)
    let q = 1_000_000_007
    type ZQ = Mod(q)
    let cq = Comb<ZQ>(10)
    assert cq.choose(10, 3) == ZQ(120)
    type Z12 = Mod<12>
    assert Z12(7) * Z12(7).inv() == Z12(1)
    assert Z12(1) ** 0 == Z12(1)

@test
fn inv_is_one_over_i():
    type Z = Mod<1000000007>
    let c = Comb<Z>()
    for i in 1..200: assert c.inv(i) * Z(i) == Z(1)
    assert c.inv(1) == Z(1)

@test
fn inv_keeps_up_as_the_factorials_grow():
    type Z = Mod<998_244_353>
    let c = Comb<Z>()
    assert c.inv(2) * Z(2) == Z(1)
    assert c.fact(3000) != Z(0)
    for i in [2999, 3000, 1, 1500]: assert c.inv(i) * Z(i) == Z(1)
    let d = Comb<Z>(10)
    for i in (1..5000).rev: assert d.inv(i) * Z(i) == Z(1)

@test
fn comb_multinomial():
    type Z = Mod<998_244_353>
    let c = Comb<Z>(20)
    assert c.choose(10, 3) == Z(120)
    assert c.multinomial([]) == Z(1)
    assert c.multinomial([5]) == Z(1)
    assert c.multinomial([2, 3, 1]) == Z(60)
    assert c.multinomial([2, -1, 3]) == Z(0)
    // Lucas fallback when total >= P
    let cs = Comb<Mod<7>>()
    assert cs.multinomial([3, 3, 3]) == cs.choose(9, 3) * cs.choose(6, 3)
    assert cs.multinomial([5, 5]) == cs.choose(10, 5)

impl Show for Comb<M>:
    /// How far its tables reach, as `factorials up to n`.
    ///
    /// example: `type Z = Mod<7>; "{Comb<Z>(3)}" == "factorials up to 3"`
    fn show(c) = "factorials up to {c.f.len - 1}"

impl Show for PrimePowerComb:
    /// Its modulus, as `choose mod p^e`.
    ///
    /// example: `"{PrimePowerComb(2, 3)}" == "choose mod 2^3"`
    fn show(c) = "choose mod {c.p}^{c.e}"
