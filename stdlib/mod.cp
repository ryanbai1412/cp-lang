//! Modular arithmetic and counting.
//!
//! `Mod<P>` is a number modulo a constant `P`: the modulus is part of the
//! type, takes no memory, and two moduli cannot be mixed by accident.
//! The modulus may be a constant (`type Z = Mod<998244353>`) or a value read from
//! the input (`let P: Int = read()`, then `type Z = Mod<P>`): one type serves both.
//! `P * P` must fit in an `Int`, so `P < 3.03e9`.
use std.algebra.{Monoid}
use std.fenwick.{Fenwick}
use std.math.{gcd}

/// An integer modulo the constant `P`, always stored in `0..P`.
/// `+ - *` are O(1); `/` and `inv` are O(log P) and need `gcd(x, P) == 1`.
pub struct Mod<P>(v: Int):
    /// The modulus: `Z.MOD`.
    let MOD = P
    let zero = Mod(0)
    let one = Mod(1 % P)

    /// Any `Int`, reduced: `Mod<7>(-1)` is 6.
    pub fn new(x: Int) -> Mod<P> = Mod(x % P)

    /// `self` to the power `e`; a negative `e` uses the inverse. O(log e).
    pub fn pow(self, e: Int) -> Mod<P>:
        if e < 0: return self.inv.pow(-e)
        var (base, acc, k) = (self, Mod(1 % P), e)
        while k > 0:
            if k % 2 == 1: acc = acc * base
            base = base * base
            k /= 2
        acc

    /// The multiplicative inverse when `gcd(v, P) == 1`, or `none` otherwise. O(log P).
    pub fn try_inv(self) -> ?Mod<P>:
        var (a, b, x, y) = (v, P, 1, 0)
        while b != 0:
            let q = a / b
            (a, b) = (b, a - q * b)
            (x, y) = (y, x - q * y)
        if a != 1: return none
        Mod(x % P)

    /// The multiplicative inverse, by the extended Euclidean algorithm (so `P` need not be prime).
    pub @get fn inv(self) -> Mod<P>:
        let x = self.try_inv()
        assert x != none, "it has no inverse: 0 never has one, and no value that shares a factor with the modulus"
        x!

impl Add for Mod<P>:
    fn add(a, b) = Mod(if a.v + b.v >= P: a.v + b.v - P else: a.v + b.v)

impl Sub for Mod<P>:
    fn sub(a, b) = Mod(if a.v >= b.v: a.v - b.v else: a.v - b.v + P)

impl Neg for Mod<P>:
    fn neg(a) = Mod(if a.v == 0: 0 else: P - a.v)

impl Mul for Mod<P>:
    fn mul(a, b) = Mod(a.v * b.v % P)

impl Div for Mod<P>:
    fn div(a, b) = a * b.inv

impl Show for Mod<P>:
    fn show(a) = "{a.v}"

/// Factorials and inverse factorials modulo a prime `P`.
/// Uses interior mutability to grow its table lazily on demand: an immutable
/// `let c = Comb<P>()` (or `Comb<P>(limit)` to pre-size) answers queries in O(1)
/// with amortized O(1) growth per new factorial.
pub struct Comb<P>:
    f: List<Mod<P>>
    g: List<Mod<P>>

    pub fn new(limit: Int = 0) -> Comb<P>:
        let c = Comb([Mod<P>(1)], [Mod<P>(1)])
        if limit > 0: c.ensure(limit)
        c

    fn ensure(self, n: Int):
        if n < f.len: return
        let old = f.len
        var cap = old
        while cap <= n: cap *= 2
        if P > 1 and cap > P: cap = P
        while f.len < cap:
            let i = f.len
            f.push(f[i - 1] * Mod<P>(i))
        let need = cap - old
        var tail = List.filled(need, Mod<P>(0))
        tail[need - 1] = f[cap - 1].inv
        for i in (0..need - 1).rev: tail[i] = tail[i + 1] * Mod<P>(old + i + 1)
        for x in tail: g.push(x)

    /// `n!`
    pub fn fact(self, n: Int) -> Mod<P>:
        assert n >= 0
        if n >= P: return Mod<P>(0)
        self.ensure(n)
        f[n]

    /// `1 / n!` (requires `0 <= n < P`).
    pub fn inv_fact(self, n: Int) -> Mod<P>:
        assert 0 <= n and n < P
        self.ensure(n)
        g[n]

    /// The number of `k`-subsets of `n` things; 0 when `k` is outside `0..=n`.
    /// Uses Lucas's theorem automatically when `n >= P` (for prime `P`).
    pub fn choose(self, n: Int, k: Int) -> Mod<P>:
        if k < 0 or k > n: return Mod<P>(0)
        if n >= P: return self.choose(n / P, k / P) * self.choose(n % P, k % P)
        self.ensure(n)
        f[n] * g[k] * g[n - k]

    /// The number of ordered selections of `k` out of `n` things.
    pub fn arrange(self, n: Int, k: Int) -> Mod<P>:
        if k < 0 or k > n: return Mod<P>(0)
        self.ensure(n)
        f[n] * g[n - k]

    /// The number of ways to put `n` equal balls into `k` boxes (stars and bars).
    pub fn multichoose(self, n: Int, k: Int) -> Mod<P>:
        if n == 0: return Mod<P>(1)
        if k <= 0 or n < 0: return Mod<P>(0)
        self.choose(n + k - 1, n)

    /// The `n`-th Catalan number.
    pub fn catalan(self, n: Int) -> Mod<P> = self.choose(2 * n, n) - self.choose(2 * n, n + 1)

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
        assert M(x).pow(998244352) == M.one
        assert M(x).pow(-1) == M(x).inv
        var p = M.one
        for e in 0..20:
            assert M(x).pow(e) == p
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
    for v in V: f.add(v, M(10).pow(9 + v.to_idx))
    assert f.sum(V(0)..=V(4)) == M(1000000000) * M(11111)

@test
fn choose_matches_pascals_triangle():
    type M = Mod<1000000007>
    // Immutable binding: Comb<P> auto-grows via interior mutability
    let c = Comb<1000000007>()
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
    let small = Comb<11>()
    assert small.choose(10, 4) == Mod<11>(210)
    assert small.fact(10) == Mod<11>(10)
    // Pre-sizing still works
    let pre = Comb<1000000007>(40)
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
        assert Z(5).pow(3) == Z(125 % p) and Z(7).pow(0) == Z(1)
    type Z12 = Mod<12>
    assert Z12(7) * Z12(7).inv == Z12(1)
    assert Z12(1).pow(0) == Z12(1)
