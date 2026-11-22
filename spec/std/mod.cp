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

    /// Any `Int`, reduced into `0..P` by Euclidean remainder: `Mod<7>(-1)` is 6.
    ///
    /// `Mod<P>(x)` calls `new`, so every way of building a `Mod` reduces.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Mod<7>.new(-1) == Mod<7>(6)`
    /// example: `Mod<7>.new(15) == Mod<7>(1)`
    /// example: `Mod<1>.new(5) == Mod<1>(0)`
    pub fn new(x: Int) -> Mod<P> = todo

    /// The multiplicative inverse when `gcd(v, P) == 1`, or `None` otherwise.
    ///
    /// Uses the extended Euclidean algorithm, so `P` need not be prime. With `P == 1` the only value
    /// is 0, which is also `one`, so `Mod<1>(0).try_inv()` is `Some(Mod<1>(0))`.
    ///
    /// cost: O(log P)
    /// traps: never
    /// example: `Mod<7>(3).try_inv() == Some(Mod<7>(5))`
    /// example: `Mod<6>(4).try_inv() == None`
    /// example: `Mod<7>(0).try_inv() == None`
    pub fn try_inv(self) -> ?Mod<P> = todo

    /// The multiplicative inverse, by the extended Euclidean algorithm (so `P` need not be prime).
    ///
    /// `x.inv()` is `x.try_inv().unwrap()` with a trap that names `x` and `P`; `a / b` is
    /// `a * b.inv()`.
    ///
    /// cost: O(log P)
    /// traps: when `gcd(v, P) != 1`
    /// example: `Mod<7>(3).inv() == Mod<7>(5)`
    /// example: `Mod<10>(3).inv() == Mod<10>(7)`
    pub fn inv(self) -> Mod<P> = todo

    /// Lift an `Int` to `Mod<P>`; the same reduction as `new`.
    ///
    /// This is the conversion an `Int` literal takes where a `Mod<P>` is expected, so
    /// `let x: Mod<7> = 10` is `Mod<7>.from_int(10)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Mod<7>.from_int(10) == Mod<7>(3)`
    /// example: `Mod<7>.from_int(-8) == Mod<7>(6)`
    pub fn from_int(x: Int) -> Mod<P> = todo

    /// A square root modulo the prime `P` (Tonelli-Shanks), or `None` when there is none.
    ///
    /// Of the two roots `r` and `P - r`, it returns the one with the smaller value `v`; the root of 0
    /// is 0.
    ///
    /// cost: O(log^2 P)
    /// ties: the root with the smaller value
    /// traps: when `P` is not prime
    /// example: `Mod<7>(2).sqrt() == Some(Mod<7>(3))`
    /// example: `Mod<7>(3).sqrt() == None`
    /// example: `Mod<7>(0).sqrt() == Some(Mod<7>(0))`
    pub fn sqrt(self) -> ?Mod<P> = todo

    /// The least `k >= 0` with `self ** k == target` (baby-step giant-step), or `None`.
    ///
    /// Any modulus works: the non-coprime part is peeled off first. `self ** 0` is `one`, so
    /// `x.log(one)` is `Some(0)` for every `x`.
    ///
    /// cost: O(sqrt P)
    /// ties: the least `k`
    /// traps: never
    /// example: `Mod<7>(3).log(Mod<7>(6)) == Some(3)`
    /// example: `Mod<7>(3).log(Mod<7>(1)) == Some(0)`
    /// example: `Mod<7>(2).log(Mod<7>(3)) == None`
    pub fn log(self, target: Mod<P>) -> ?Int = todo

    /// The inverse of every element of `xs`, with one `inv` in all (Montgomery's trick).
    ///
    /// cost: O(n + log P)
    /// empty: `[]`
    /// order: same order as `xs`; a new `List`, keyed from 0
    /// traps: when any element has `gcd(v, P) != 1` (in particular 0)
    /// example: `Mod<7>.inv_all([Mod<7>(1), Mod<7>(2), Mod<7>(3)]) == [Mod<7>(1), Mod<7>(4), Mod<7>(5)]`
    /// example: `let e: List<Mod<7>> = []; Mod<7>.inv_all(e) == []`
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
/// Its tables are `@cache` fields that grow on demand, so an immutable
/// `let c = Comb<Z>()` (or `Comb<Z>(limit)` to pre-size) answers queries in O(1)
/// with amortized O(1) growth per new factorial.
pub struct Comb<M>:
    @cache f: List<M>
    @cache g: List<M>

    /// Precompute factorials and inverse factorials up to `limit` for `M = Mod<P>`, `P` prime.
    ///
    /// The tables hold `min(limit, P - 1) + 1` entries and grow on demand later (doubling, capped at
    /// `P`), so `Comb<Z>()` with no limit is valid and simply grows on first use.
    ///
    /// cost: O(min(limit, P))
    /// traps: when `limit < 0`, or `P` is not prime
    /// example: `type Z = Mod<7>; let c = Comb<Z>.new(3); c.fact(3) == Z(6)`
    pub fn new(limit: Int = 0) -> Comb<M> = todo

    fn grow(self, want: Int) = todo

    fn ensure(self, n: Int) = todo

    /// `n!`, reduced; 0 when `n >= M.MOD` (the product contains `P`).
    ///
    /// cost: O(1) amortized
    /// traps: when `n < 0`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>.new().fact(5) == Z(120)`
    /// example: `type Z = Mod<7>; Comb<Z>.new().fact(6) == Z(6)`
    /// example: `type Z = Mod<7>; Comb<Z>.new().fact(7) == Z(0)`
    pub fn fact(self, n: Int) -> M = todo

    /// `1 / n!`, for `0 <= n < M.MOD`.
    ///
    /// cost: O(1) amortized
    /// traps: when `n` is outside `0..M.MOD`
    /// example: `type Z = Mod<7>; Comb<Z>.new().inv_fact(3) == Z(6)`
    /// example: `type Z = Mod<7>; Comb<Z>.new().inv_fact(0) == Z(1)`
    pub fn inv_fact(self, n: Int) -> M = todo

    /// `1 / i`, for `1 <= i < M.MOD`, from the factorials: `(i - 1)! / i!`.
    ///
    /// cost: O(1) amortized
    /// traps: when `i` is outside `1..M.MOD`
    /// example: `type Z = Mod<7>; Comb<Z>.new().inv(3) == Z(5)`
    pub fn inv(self, i: Int) -> M = todo

    fn choose_lucas(self, n: Int, k: Int) -> M = todo

    /// The number of `k`-subsets of `n` things, `n! / (k! (n - k)!)`; 0 when `k` is outside `0..=n`.
    ///
    /// For `n < M.MOD` it reads the tables. For `n >= M.MOD` it uses Lucas's theorem on the base-`P`
    /// digits, so the answer is exact modulo `P` for every `n`. A negative `n` gives 0.
    ///
    /// cost: O(1) amortized for `n < M.MOD`; O(log_P n) otherwise
    /// traps: never
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>.new().choose(5, 2) == Z(10)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>.new().choose(5, 6) == Z(0)`
    /// example: `type Z = Mod<7>; Comb<Z>.new().choose(10, 3) == Z(1)`
    /// example: `type Z = Mod<7>; Comb<Z>.new().choose(7, 1) == Z(0)`
    pub fn choose(self, n: Int, k: Int) -> M = todo

    /// The number of ordered selections of `k` out of `n` things, `n! / (n - k)!`; 0 when `k` is
    /// outside `0..=n`.
    ///
    /// The value is the falling factorial `n (n - 1) ... (n - k + 1)` reduced modulo `P`: 0 when
    /// those `k` numbers include a multiple of `P`, else `fact(n % P) / fact((n - k) % P)`, so it is
    /// exact for `n >= M.MOD` too.
    ///
    /// cost: O(1) amortized
    /// traps: never
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>.new().arrange(5, 2) == Z(20)`
    /// example: `type Z = Mod<7>; Comb<Z>.new().arrange(9, 2) == Z(2)`
    /// example: `type Z = Mod<7>; Comb<Z>.new().arrange(8, 2) == Z(0)`
    pub fn arrange(self, n: Int, k: Int) -> M = todo

    /// The number of ways to put `n` equal balls into `k` boxes (stars and bars): `choose(n + k - 1, n)`.
    ///
    /// `multichoose(0, 0)` is 1 (the empty placement) and `multichoose(n, 0)` is 0 for `n > 0`; a
    /// negative `n` or `k` gives 0.
    ///
    /// cost: O(1) amortized
    /// traps: never
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>.new().multichoose(3, 2) == Z(4)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>.new().multichoose(2, 3) == Z(6)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>.new().multichoose(0, 0) == Z(1)`
    pub fn multichoose(self, n: Int, k: Int) -> M = todo

    /// The `n`-th Catalan number, `choose(2n, n) - choose(2n, n + 1)`.
    ///
    /// The difference form needs no division by `n + 1`, so it stays exact when `n + 1` is a
    /// multiple of `P`.
    ///
    /// cost: O(1) amortized
    /// traps: when `n < 0`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>.new().catalan(0) == Z(1)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>.new().catalan(4) == Z(14)`
    /// example: `type Z = Mod<7>; Comb<Z>.new().catalan(4) == Z(0)`
    pub fn catalan(self, n: Int) -> M = todo

    /// The number of derangements `!n`: permutations of `n` things with no fixed point.
    ///
    /// `!0 = 1`, `!1 = 0`, and `!n = (n - 1)(!(n - 1) + !(n - 2))`; the values are tabled like the
    /// factorials.
    ///
    /// cost: O(1) amortized, O(n) table
    /// traps: when `n < 0`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>.new().derange(0) == Z(1)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>.new().derange(4) == Z(9)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>.new().derange(5) == Z(44)`
    pub fn derange(self, n: Int) -> M = todo

    /// The multinomial coefficient `(sum ks)! / prod(k!)`; 0 if any `k < 0`.
    ///
    /// It is computed as the product of `choose(k_1 + ... + k_i, k_i)`, so Lucas applies and the
    /// result is exact modulo `P` even when `sum ks >= M.MOD`.
    ///
    /// cost: O(ks.len) amortized
    /// empty: 1 (the empty multinomial)
    /// traps: never
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>.new().multinomial([2, 1]) == Z(3)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>.new().multinomial([1, 1, 1]) == Z(6)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>.new().multinomial([2, -1]) == Z(0)`
    /// example: `type Z = Mod<1_000_000_007>; Comb<Z>.new().multinomial([]) == Z(1)`
    pub fn multinomial(self, ks: List<Int>) -> M = todo

/// O(1) reduction. Fast Barrett modular multiplier for a runtime modulus `1 <= m < 2^31`.
pub struct Barrett(m: Int, im: U64):
    /// A Barrett multiplier for the runtime modulus `m`, `1 <= m < 2^31`: precomputes
    /// `im = floor((2^64 - 1) / m) + 1` (wrapping, so 0 for `m == 1`).
    ///
    /// cost: O(1)
    /// traps: when `m` is outside `1..2^31`
    /// example: `Barrett.new(7).mul(3, 5) == 1`
    pub fn new(m: Int) -> Barrett = todo

    /// `(a * b) % m` without a hardware division, for `a, b` in `0..m`.
    ///
    /// cost: O(1)
    /// traps: when `a` or `b` is outside `0..m`
    /// example: `Barrett.new(7).mul(3, 5) == 1`
    /// example: `Barrett.new(1_000_000_007).mul(1_000_000_006, 1_000_000_006) == 1`
    /// example: `Barrett.new(1).mul(0, 0) == 0`
    pub fn mul(self, a: Int, b: Int) -> Int = todo

    /// `a % m`, for `a < m * m`.
    ///
    /// cost: O(1)
    /// traps: when `a >= m * m`
    /// example: `Barrett.new(7).reduce(48) == 6`
    /// example: `Barrett.new(7).reduce(0) == 0`
    pub fn reduce(self, a: U64) -> Int = todo

/// O(p^e) build, O(log_p n) query. Binomial coefficients modulo a prime power `p^e` (Granville's theorem).
pub struct PrimePowerComb(p: Int, e: Int, pe: Int):
    /// Tables for binomial coefficients modulo `p^e`: the factorials with every factor of `p`
    /// removed, modulo `p^e`.
    ///
    /// cost: O(p^e) time and memory
    /// traps: when `p` is not prime, or `e < 1`, or `p^e` overflows `Int`
    /// example: `PrimePowerComb.new(2, 3).choose(6, 3) == 4`
    pub fn new(p: Int, e: Int) -> PrimePowerComb = todo

    /// `C(n, k) mod p^e`, by Granville's generalization of Lucas's theorem; 0 when `k` is outside
    /// `0..=n`.
    ///
    /// cost: O(log_p n)
    /// traps: never
    /// example: `PrimePowerComb.new(2, 3).choose(6, 3) == 4`
    /// example: `PrimePowerComb.new(3, 2).choose(9, 3) == 3`
    /// example: `PrimePowerComb.new(3, 2).choose(5, 7) == 0`
    pub fn choose(self, n: Int, k: Int) -> Int = todo

/// O(sum p_i^e_i) build, O(omega(m) log n) query. Binomial coefficients modulo any `m >= 1` via CRT.
pub struct ModComb(m: Int):
    /// Binomial coefficients modulo any `m >= 1`: factor `m` and build one `PrimePowerComb` per
    /// prime power, combined by CRT.
    ///
    /// cost: O(sum p_i^e_i)
    /// traps: when `m < 1`
    /// example: `ModComb.new(12).choose(6, 3) == 8`
    pub fn new(m: Int) -> ModComb = todo

    /// `C(n, k) mod m`, by CRT over the prime powers of `m`; 0 when `k` is outside `0..=n`.
    ///
    /// With `m == 1` every answer is 0.
    ///
    /// cost: O(omega(m) log n)
    /// traps: never
    /// example: `ModComb.new(12).choose(6, 3) == 8`
    /// example: `ModComb.new(10).choose(5, 2) == 0`
    /// example: `ModComb.new(1).choose(5, 2) == 0`
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
