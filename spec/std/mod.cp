//! Modular arithmetic and counting.
//!
//! `Mod<P>` is a number modulo a constant `P`: the modulus is part of the
//! type, takes no memory, and two moduli cannot be mixed by accident.
//! The modulus may be a constant (`type Z = Mod<998244353>`) or a value read from
//! the input (`let P: Int = read()`, then `type Z = Mod<P>`): one type serves both.
//! `P` may be anything below 2^62 (so `a + b` fits): above 3.03e9, where `P * P` no longer fits
//! an `Int`, `*` multiplies in 128 bits.
use std.algebra.{Monoid}
use std.fenwick.{Fenwick}
use std.math.{gcd}

/// An integer modulo the constant `P`, always stored in `0..P`.
/// `+ - *` are O(1); `/` and `inv` are O(log P) and need `gcd(x, P) == 1`.
pub struct Mod<P>(v: 0..P):
    /// The modulus: `Z.MOD`.
    let MOD = P
    let zero = Mod(0)
    let one = Mod(1 % P)

    /// Any `Int`, reduced: `Mod<7>(-1)` is 6.
    pub fn new(x: Int) -> Mod<P> = Mod(x % P)

    /// The multiplicative inverse when `gcd(v, P) == 1`, or `none` otherwise. O(log P).
    pub fn try_inv(self) -> ?Mod<P>:
        var (a, b, x, y) = (v, P, 1, 0)
        while b != 0:
            let q = a / b
            (a, b) = (b, a - q * b)
            (x, y) = (y, x - q * y)
        if a != 1: return none
        Mod(x % P)

    /// O(log P). The multiplicative inverse, by the extended Euclidean algorithm (so `P` need not be prime).
    pub @get fn inv(self) -> Mod<P>:
        let x = self.try_inv()
        assert x != none, "it has no inverse: 0 never has one, and no value that shares a factor with the modulus"
        x!

    /// O(1). Lift an `Int` literal to `Mod<P>`.
    pub fn from_int(x: Int) -> Mod<P> = Mod(x % P)

    /// O(log^2 P). Square root mod prime `P` (Tonelli-Shanks), or `none`.
    pub fn sqrt(self) -> ?Mod<P> = todo

    /// O(sqrt P). Discrete log `k >= 0` with `self ** k == target` (BSGS), or `none`.
    pub fn log(self, target: Mod<P>) -> ?Int = todo

    /// O(n + log P). Batch modular inverses of all elements in `xs` (Montgomery's trick); traps on 0.
    pub fn inv_all(xs: List<Mod<P>>) -> List<Mod<P>> = todo

impl Add for Mod<P>:
    fn add(a, b) = Mod(if a.v + b.v >= P: a.v + b.v - P else: a.v + b.v)

impl Sub for Mod<P>:
    fn sub(a, b) = Mod(if a.v >= b.v: a.v - b.v else: a.v - b.v + P)

impl Neg for Mod<P>:
    fn neg(a) = Mod(if a.v == 0: 0 else: P - a.v)

impl Mul for Mod<P>:
    // The product of two residues fits an `Int` only below 3_037_000_500; a larger modulus
    // multiplies in 128 bits. With a constant `P` the test is settled when the C++ is built.
    fn mul(a, b) = Mod(
        if P <= 3_037_000_499: a.v * b.v % P else: Int(I128(a.v) * I128(b.v) % I128(P))
    )

impl Div for Mod<P>:
    fn div(a, b) = a * b.inv

impl Pow for Mod<P>:
    fn pow(a, e: Int) -> Mod<P>:
        if e < 0: return a.inv ** (-e)
        var (base, acc, k) = (a, Mod(1 % P), e)
        while k > 0:
            if k % 2 == 1: acc = acc * base
            base = base * base
            k /= 2
        acc

impl Show for Mod<P>:
    fn show(a) = "{a.v}"

/// Factorials and inverse factorials for a modular type `M = Mod<P>` (prime `P`).
/// Uses interior mutability to grow its table lazily on demand: an immutable
/// `let c = Comb<Z>()` (or `Comb<Z>(limit)` to pre-size) answers queries in O(1)
/// with amortized O(1) growth per new factorial.
pub struct Comb<M>:
    f: List<M>
    g: List<M>

    /// Precompute factorials and inverse factorials up to `limit`. O(limit).
    pub fn new(limit: Int = 0) -> Comb<M>:
        let p = M.MOD
        var cap = limit + 1
        if cap < 1: cap = 1
        if p > 1 and cap > p: cap = p
        var f = List.filled(cap, M(1))
        var g = List.filled(cap, M(1))
        unchecked:
            for i in 1..cap: f[i] = f[i - 1] * M(i)
            g[cap - 1] = f[cap - 1].inv
            for i in (1..cap).rev: g[i - 1] = g[i] * M(i)
        Comb(f, g)

    fn grow(self, want: Int):
        let p = M.MOD
        var cap = want
        if p > 1 and cap > p: cap = p
        let old = f.len
        if cap <= old: return
        f.reserve(cap)
        g.reserve(cap)
        unchecked:
            while f.len < cap:
                let i = f.len
                f.push(f[i - 1] * M(i))
            let need = cap - old
            var tail = List.filled(need, M(0))
            tail[need - 1] = f[cap - 1].inv
            for i in (0..need - 1).rev: tail[i] = tail[i + 1] * M(old + i + 1)
            for x in tail: g.push(x)

    fn ensure(self, n: Int):
        if n < f.len: return
        var cap = f.len
        while cap <= n: cap *= 2
        self.grow(cap)

    /// `n!`
    pub fn fact(self, n: Int) -> M:
        assert n >= 0
        if n >= M.MOD: return M(0)
        self.ensure(n)
        unchecked: return f[n]

    /// `1 / n!` (requires `0 <= n < M.MOD`).
    pub fn inv_fact(self, n: Int) -> M:
        assert 0 <= n and n < M.MOD
        self.ensure(n)
        unchecked: return g[n]

    /// `1 / i` (requires `1 <= i < M.MOD`), from the factorials: `(i - 1)! / i!`. O(1).
    pub fn inv(self, i: Int) -> M:
        assert 1 <= i and i < M.MOD, "`inv` needs 1 <= i < P"
        self.ensure(i)
        unchecked: return f[i - 1] * g[i]

    fn choose_lucas(self, n: Int, k: Int) -> M:
        let p = M.MOD
        self.choose(n / p, k / p) * self.choose(n % p, k % p)

    /// The number of `k`-subsets of `n` things; 0 when `k` is outside `0..=n`.
    /// Uses Lucas's theorem automatically when `n >= M.MOD` (for prime `P`).
    pub fn choose(self, n: Int, k: Int) -> M:
        if k < 0 or k > n: return M(0)
        if n >= M.MOD: return self.choose_lucas(n, k)
        self.ensure(n)
        unchecked: return f[n] * g[k] * g[n - k]

    /// The number of ordered selections of `k` out of `n` things.
    pub fn arrange(self, n: Int, k: Int) -> M:
        if k < 0 or k > n: return M(0)
        self.ensure(n)
        unchecked: return f[n] * g[n - k]

    /// The number of ways to put `n` equal balls into `k` boxes (stars and bars).
    pub fn multichoose(self, n: Int, k: Int) -> M:
        if n == 0: return M(1)
        if k <= 0 or n < 0: return M(0)
        self.choose(n + k - 1, n)

    /// The `n`-th Catalan number.
    pub fn catalan(self, n: Int) -> M = self.choose(2 * n, n) - self.choose(2 * n, n + 1)

    /// O(1) amortized, O(n) table. Number of derangements `!n` (permutations with no fixed point).
    pub fn derange(self, n: Int) -> M = todo

    /// The multinomial coefficient `(sum ks)! / prod(k!)`; 0 if any `k < 0`. O(ks.len).
    pub fn multinomial(self, ks: List<Int>) -> M:
        var total = 0
        for k in ks:
            if k < 0: return M(0)
            total += k
        if total >= M.MOD:
            var acc = M(1)
            var rem = total
            for k in ks:
                acc = acc * self.choose(rem, k)
                rem -= k
            return acc
        self.ensure(total)
        var acc = f[total]
        for k in ks: acc = acc * g[k]
        acc

/// O(1) reduction. Fast Barrett modular multiplier for a runtime modulus `1 <= m < 2^31`.
pub struct Barrett(m: Int, im: U64):
    /// O(1). Precompute Barrett reciprocal for `1 <= m < 2^31`.
    pub fn new(m: Int) -> Barrett = todo

    /// O(1). Return `(a * b) % m` without hardware division.
    pub fn mul(self, a: Int, b: Int) -> Int = todo

    /// O(1). Return `a % m` for `0 <= a < m * m`.
    pub fn reduce(self, a: U64) -> Int = todo

/// O(p^e) build, O(log_p n) query. Binomial coefficients modulo a prime power `p^e` (Granville's theorem).
pub struct PrimePowerComb(p: Int, e: Int, pe: Int):
    /// O(p^e) time and memory. Precompute prime-free factorials modulo `p^e`.
    pub fn new(p: Int, e: Int) -> PrimePowerComb = todo

    /// O(log_p n). Return `C(n, k) mod p^e`.
    pub fn choose(self, n: Int, k: Int) -> Int = todo

/// O(sum p_i^e_i) build, O(omega(m) log n) query. Binomial coefficients modulo any `m >= 1` via CRT.
pub struct ModComb(m: Int):
    /// O(sum p_i^e_i). Factor `m` and build a `PrimePowerComb` per prime power.
    pub fn new(m: Int) -> ModComb = todo

    /// O(omega(m) log n). Return `C(n, k) mod m`.
    pub fn choose(self, n: Int, k: Int) -> Int = todo

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
        assert M(x) * M(x).inv == M.one
        assert M(x) ** 998244352 == M.one
        assert M(x) ** (-1) == M(x).inv
        var p = M.one
        for e in 0..20:
            assert M(x) ** e == p
            p = p * M(x)
    // a composite modulus still has inverses for coprime values
    assert Mod<10>(3).inv == Mod<10>(7)

@test
fn mod_values_work_in_generic_structures():
    type M = Mod<1000000007>
    let xs = [M(0), M(1), M(5), M(-3), M(123456789)]
    assert laws_hold(Monoid<M>.of(+), xs)
    assert laws_hold(Monoid<M>.of(*), xs)
    domain V = 0..5
    var f = Fenwick<V, M>()
    for v in V: f.add(v, M(10) ** (9 + v.to_idx))
    assert f.sum(V(0)..=V(4)) == M(1000000000) * M(11111)

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
    assert Mod<10>(2).try_inv() == none
    assert Mod<10>(0).try_inv() == none
    assert Mod<1>(0) == Mod<1>(42) and Mod<1>(0).try_inv() == Mod<1>(0)
    assert Mod<1>.zero == Mod<1>.one

@test
fn a_modulus_read_at_run_time():
    for p in [2, 12, 1_000_000_007]:
        let P = p  // a modulus is frozen under a capitalized name
        type Z = Mod<P>
        assert Z(p) == Z(0) and Z(-1) + Z(1) == Z(0)
        assert Z(5) ** 3 == Z(125 % p) and Z(7) ** 0 == Z(1)
    let Q = 1_000_000_007
    type ZQ = Mod<Q>
    let cq = Comb<ZQ>(10)
    assert cq.choose(10, 3) == ZQ(120)
    type Z12 = Mod<12>
    assert Z12(7) * Z12(7).inv == Z12(1)
    assert Z12(1) ** 0 == Z12(1)

@test
fn inv_is_one_over_i():
    type Z = Mod<1000000007>
    let c = Comb<Z>()
    for i in 1..200: assert c.inv(i) * Z(i) == Z(1)
    assert c.inv(1) == Z(1)

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
