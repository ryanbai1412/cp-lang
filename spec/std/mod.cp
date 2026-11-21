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
    pub fn new(x: Int) -> Mod<P> = todo

    /// The multiplicative inverse when `gcd(v, P) == 1`, or `none` otherwise. O(log P).
    pub fn try_inv(self) -> ?Mod<P> = todo

    /// O(log P). The multiplicative inverse, by the extended Euclidean algorithm (so `P` need not be prime).
    pub fn inv(self) -> Mod<P> = todo

    /// O(1). Lift an `Int` literal to `Mod<P>`.
    pub fn from_int(x: Int) -> Mod<P> = todo

    /// O(log^2 P). Square root mod prime `P` (Tonelli-Shanks), or `none`.
    pub fn sqrt(self) -> ?Mod<P> = todo

    /// O(sqrt P). Discrete log `k >= 0` with `self ** k == target` (BSGS), or `none`.
    pub fn log(self, target: Mod<P>) -> ?Int = todo

    /// O(n + log P). Batch modular inverses of all elements in `xs` (Montgomery's trick); traps on 0.
    pub fn inv_all(xs: List<Mod<P>>) -> List<Mod<P>> = todo

impl Add for Mod<P>:
    fn add(a, b) = todo

impl Sub for Mod<P>:
    fn sub(a, b) = todo

impl Neg for Mod<P>:
    fn neg(a) = todo

impl Mul for Mod<P>:
    // The product of two residues fits an `Int` only below 3_037_000_500; a larger modulus
    // multiplies in 128 bits. With a constant `P` the test is settled when the C++ is built.
    fn mul(a, b) = todo

impl Div for Mod<P>:
    fn div(a, b) = todo

impl Pow for Mod<P>:
    fn pow(a, e: Int) -> Mod<P> = todo

impl Show for Mod<P>:
    fn show(a) = todo

/// Factorials and inverse factorials for a modular type `M = Mod<P>` (prime `P`).
/// Uses interior mutability to grow its table lazily on demand: an immutable
/// `let c = Comb<Z>()` (or `Comb<Z>(limit)` to pre-size) answers queries in O(1)
/// with amortized O(1) growth per new factorial.
pub struct Comb<M>:
    f: List<M>
    g: List<M>

    /// Precompute factorials and inverse factorials up to `limit`. O(limit).
    pub fn new(limit: Int = 0) -> Comb<M> = todo

    fn grow(self, want: Int) = todo

    fn ensure(self, n: Int) = todo

    /// `n!`
    pub fn fact(self, n: Int) -> M = todo

    /// `1 / n!` (requires `0 <= n < M.MOD`).
    pub fn inv_fact(self, n: Int) -> M = todo

    /// `1 / i` (requires `1 <= i < M.MOD`), from the factorials: `(i - 1)! / i!`. O(1).
    pub fn inv(self, i: Int) -> M = todo

    fn choose_lucas(self, n: Int, k: Int) -> M = todo

    /// The number of `k`-subsets of `n` things; 0 when `k` is outside `0..=n`.
    /// Uses Lucas's theorem automatically when `n >= M.MOD` (for prime `P`).
    pub fn choose(self, n: Int, k: Int) -> M = todo

    /// The number of ordered selections of `k` out of `n` things.
    pub fn arrange(self, n: Int, k: Int) -> M = todo

    /// The number of ways to put `n` equal balls into `k` boxes (stars and bars).
    pub fn multichoose(self, n: Int, k: Int) -> M = todo

    /// The `n`-th Catalan number.
    pub fn catalan(self, n: Int) -> M = todo

    /// O(1) amortized, O(n) table. Number of derangements `!n` (permutations with no fixed point).
    pub fn derange(self, n: Int) -> M = todo

    /// The multinomial coefficient `(sum ks)! / prod(k!)`; 0 if any `k < 0`. O(ks.len).
    pub fn multinomial(self, ks: List<Int>) -> M = todo

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
fn values_stay_reduced() = todo

@test
fn pow_and_inverse() = todo

@test
fn mod_values_work_in_generic_structures() = todo

@test
fn choose_matches_pascals_triangle() = todo

@test
fn try_inv_and_modulus_one() = todo

@test
fn a_modulus_read_at_run_time() = todo

@test
fn inv_is_one_over_i() = todo

@test
fn comb_multinomial() = todo
