//! Formal power series operations over NTT-friendly `Mod<P>`.
//!
//! Series are plain `List<Mod<P>>` (lowest degree first: `a[k]` is `[x^k] a(x)`),
//! matching `std.ntt.convolve`. Every truncated operation takes an explicit output
//! length `n >= 0` (computing modulo `x^n`, returning length `n`; `n == 0` returns `[]`).
use std.math.{mod_sqrt}
use std.mod.{Mod}
use std.ntt.{convolve, ntt}

/// Formal derivative `a'(x)`. Length `max(0, a.len - 1)`. O(a.len).
pub fn deriv<P>(a: List<Mod<P>>) -> List<Mod<P>> = todo

/// Formal antiderivative with constant term 0. Length `a.len + 1` (or `[]` if empty).
/// Uses linear-time modular inverses `1..=a.len`. O(a.len).
pub fn integral<P>(a: List<Mod<P>>) -> List<Mod<P>> = todo

/// Multiplicative inverse `a(x)^(-1) mod x^n` via half-size Newton doubling.
/// Requires `n >= 0`, and when `n > 0`, `a.len > 0` and `a[0] != 0`. O(n log n).
pub fn inv<P>(a: List<Mod<P>>, n: Int) -> List<Mod<P>> = todo

/// Formal logarithm `ln(a(x)) mod x^n = integral(a'(x) / a(x)) mod x^n`.
/// Requires `n >= 0`, and when `n > 0`, `a.len > 0` and `a[0] == 1`. O(n log n).
pub fn log<P>(a: List<Mod<P>>, n: Int) -> List<Mod<P>> = todo

/// Formal exponential `exp(a(x)) mod x^n` via Newton doubling.
/// Requires `n >= 0`, and when `n > 0`, `a.is_empty` or `a[0] == 0`. O(n log n).
pub fn exp<P>(a: List<Mod<P>>, n: Int) -> List<Mod<P>> = todo

/// Formal power `a(x)^k mod x^n` (supports `k < 0` when `a[0] != 0`). O(n log n).
pub fn pow<P>(a: List<Mod<P>>, k: Int, n: Int) -> List<Mod<P>> = todo

/// Formal square root `g(x)` with `g(x)^2 == a(x) (mod x^n)`, or `None` if none exists mod `P`.
/// Requires `n >= 0`. O(n log n + log^2 P).
pub fn sqrt<P>(a: List<Mod<P>>, n: Int) -> ?List<Mod<P>> = todo

/// `self(x)` by Horner's rule, over any ring (`Mod<P>`, `Int`, `F64`, `Mat`): `coef.eval(x)`
/// with `coef[k]` the coefficient of `x^k`. O(self.len).
pub fn eval<S: Indexed, T>(self: S, x: T) -> T = todo

fn trim_poly<P>(a: List<Mod<P>>) -> List<Mod<P>> = todo

fn rev_poly<P>(a: List<Mod<P>>) -> List<Mod<P>> = todo

fn middle_product<P>(a: List<Mod<P>>, b: List<Mod<P>>) -> List<Mod<P>> = todo

/// Euclidean polynomial division `(q, r)` with `a = q * b + r` and `r.len < b.len`.
/// Requires `b` to be non-zero. O(a.len log a.len).
pub fn div_mod<P>(a: List<Mod<P>>, b: List<Mod<P>>) -> (q: List<Mod<P>>, r: List<Mod<P>>) = todo

fn subproduct_tree<P>(xs: List<Mod<P>>) -> List<List<Mod<P>>> = todo

/// Monic polynomial `prod_{r in roots} (x - r)`. O(n log^2 n).
pub fn prod_linear<P>(roots: List<Mod<P>>) -> List<Mod<P>> = todo

/// Taylor shift `a(x + c)`. Length `a.len`. O(n log n).
pub fn taylor_shift<P>(a: List<Mod<P>>, c: Mod<P>) -> List<Mod<P>> = todo

/// Multipoint evaluation `[a(x) for x in xs]` via transposed subproduct tree.
/// O((a.len + xs.len) log^2 xs.len).
pub fn eval_all<P>(a: List<Mod<P>>, xs: List<Mod<P>>) -> List<Mod<P>> = todo

fn biv_advance<P>(mut c: List<Mod<P>>, l_big: Int, l_step: Int) -> List<Mod<P>> = todo

/// Formal power series composition `f(g(x)) mod x^n` via Kinoshita-Li.
/// Handles any `g[0]` (shifts via `taylor_shift` when `g[0] != 0`). O(n log^2 n).
pub fn compose<P>(f: List<Mod<P>>, g: List<Mod<P>>, n: Int) -> List<Mod<P>> = todo

/// Value at `x` of the unique polynomial of degree `< ys.len` passing through `(i, ys[i])`
/// for `i in 0..ys.len`. Works for any prime `P` using one modular inverse. O(ys.len).
pub fn lagrange_at<P>(ys: List<Mod<P>>, x: Mod<P>) -> Mod<P> = todo

/// Unique polynomial `f` of degree `< xs.len` with `f(xs[i]) = ys[i]` for distinct `xs`.
/// O(n^2) for `n <= 64` (any prime `P`), O(n log^2 n) via subproduct tree + NTT for `n > 64`.
pub fn interpolate<P>(xs: List<Mod<P>>, ys: List<Mod<P>>) -> List<Mod<P>> = todo

/// O(n) storage. Algebraic polynomial / formal power series wrapper over `T` (lowest degree first).
pub struct Poly<T>(c: List<T>):
    /// O(n). Construct a polynomial from coefficients `c`.
    pub fn new(c: List<T>) -> Poly<T> = todo

    /// O(1). Constant polynomial `v`.
    pub fn from_int(v: Int) -> Poly<T> = todo

    /// O(1). Zero polynomial `0`.
    pub @get fn zero() -> Poly<T> = todo

    /// O(1). Constant polynomial `1`.
    pub @get fn one() -> Poly<T> = todo

    /// O(1). Indeterminate `x`.
    pub @get fn x() -> Poly<T> = todo

    /// O(1). Number of stored coefficients.
    pub @get fn len(self) -> Int = todo

    /// O(1). Degree (`-1` for the zero polynomial).
    pub @get fn deg(self) -> Int = todo

    /// O(1). Whether all coefficients are zero.
    pub @get fn is_zero(self) -> Bool = todo

    /// O(1). Read coefficient `i` (0 when `i >= self.len`).
    pub fn at(self, i: Int) -> T = todo

    /// O(1). Write coefficient `i`, growing with zeros if needed.
    pub fn set(mut self, i: Int, v: T) = todo

    /// O(n). Formal derivative `f'(x)`.
    pub fn deriv(self) -> Poly<T> = todo

    /// O(n). Formal integral with zero constant term.
    pub fn integral(self) -> Poly<T> = todo

    /// O((n + m) log(n + m)). Transposed (middle) product: `out[i] = sum_j self[i + j] * b[j]`.
    pub fn transposed_mul(self, b: Poly<T>) -> Poly<T> = todo

    /// O(n). Truncate or zero-pad to length `n` (mod `x^n`).
    pub fn truncated(self, n: Int) -> Poly<T> = todo

    /// O(n log n). Multiplicative inverse mod `x^n` (`self[0] != 0`).
    pub fn inv(self, n: Int) -> Poly<T> = todo

    /// O(n log n). Formal logarithm mod `x^n` (`self[0] == 1`).
    pub fn log(self, n: Int) -> Poly<T> = todo

    /// O(n log n). Formal exponential mod `x^n` (`self[0] == 0`).
    pub fn exp(self, n: Int) -> Poly<T> = todo

    /// O(n log n). Formal power `self(x)^k mod x^n`.
    pub fn pow(self, k: Int, n: Int) -> Poly<T> = todo

    /// O(n log n + log^2 P). Formal square root mod `x^n`, or `None`.
    pub fn sqrt(self, n: Int) -> ?Poly<T> = todo

    /// O(n). Evaluate at `x` by Horner's rule.
    pub fn eval(self, x: T) -> T = todo

    /// O(n log n). Taylor shift `self(x + c)`.
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

    /// O(m log^2 m). Build the subproduct tree for `xs`; optional `keep` truncates products mod `x^keep`.
    pub fn new(xs: List<Mod<P>>, keep: Int = -1) -> SubproductTree<P> = todo

    /// O((deg f + m) log^2 m). Evaluate `f` at every point in the tree.
    pub fn eval(self, f: List<Mod<P>>) -> List<Mod<P>> = todo

    /// O(m log^2 m). Transposed multipoint evaluation (Bostan-Lecerf-Schost / Tellegen's principle).
    pub fn eval_transposed(self, c: List<Mod<P>>) -> List<Mod<P>> = todo

    /// O(m log^2 m). Unique degree `< m` polynomial taking values `ys` at the tree's points.
    pub fn interpolate(self, ys: List<Mod<P>>) -> List<Mod<P>> = todo

/// O(k log k log n) for `k = max(p.len, q.len)`. Exact coefficient `[x^n] (P(x) / Q(x))` via Bostan-Mori (`q[0] != 0`).
pub fn bostan_mori<P>(p: List<Mod<P>>, q: List<Mod<P>>, n: Int) -> Mod<P> = todo

/// O((n + m) log(n + m)). Transposed (middle) product: `out[i] = sum_j a[i + j] * b[j]`.
pub fn transposed_mul<P>(a: List<Mod<P>>, b: List<Mod<P>>) -> List<Mod<P>> = todo

/// O(n log^2 n). Compositional inverse `g` with `f(g(x)) == x (mod x^n)` (`f[0] == 0, f[1] != 0`).
pub fn compose_inv<P>(f: List<Mod<P>>, n: Int) -> List<Mod<P>> = todo

/// O(d log d log k). Polynomial power `a(x)^k mod mod_p(x)` where `d = mod_p.len`.
pub fn pow_mod_poly<P>(a: List<Mod<P>>, k: Int, mod_p: List<Mod<P>>) -> List<Mod<P>> = todo

/// O(n log^2 n). Monic polynomial GCD via half-GCD.
pub fn gcd_poly<P>(a: List<Mod<P>>, b: List<Mod<P>>) -> List<Mod<P>> = todo

/// O((n + m) log(n + m)). Bluestein / Chirp-Z evaluation `[a(r^0), a(r^1), ..., a(r^(m - 1))]`.
pub fn chirp_z<P>(a: List<Mod<P>>, r: Mod<P>, m: Int) -> List<Mod<P>> = todo

/// O((n + m) log(n + m)). Given `f(0..n)` for `deg f < n`, return `[f(c), f(c + 1), ..., f(c + m - 1)]`.
pub fn sample_points_shift<P>(ys: List<Mod<P>>, c: Mod<P>, m: Int) -> List<Mod<P>> = todo

/// O((n + deg) log^2 n). Power sums `sum_i xs[i]^k` for `k in 0..=deg` via `-x p'(x) / p(x)`.
pub fn power_sums<P>(xs: List<Mod<P>>, deg: Int) -> List<Mod<P>> = todo

/// O(n log n). Unsigned Stirling numbers of the first kind `s(n, 0..=n)`.
pub fn stirling1_row<P>(n: Int) -> List<Mod<P>> = todo

/// O(n log n). Stirling numbers of the second kind `S(n, 0..=n)`.
pub fn stirling2_row<P>(n: Int) -> List<Mod<P>> = todo

/// O(n log n). Unsigned Stirling numbers of the first kind column `s(0..=n, k)`.
pub fn stirling1_col<P>(k: Int, n: Int) -> List<Mod<P>> = todo

/// O(n log n). Stirling numbers of the second kind column `S(0..=n, k)`.
pub fn stirling2_col<P>(k: Int, n: Int) -> List<Mod<P>> = todo

/// O(n log n). Bell numbers `B_0..=B_n`.
pub fn bell_table<P>(n: Int) -> List<Mod<P>> = todo

/// O(n log n). Bernoulli numbers `B_0..=B_n` (`B_1 = -1/2`).
pub fn bernoulli_table<P>(n: Int) -> List<Mod<P>> = todo

/// O(n log n). Partition numbers `p(0..=n)` via Euler's pentagonal theorem.
pub fn partition_table<P>(n: Int) -> List<Mod<P>> = todo

@test
fn eval_div_shift_mpe_laws() = todo

@test
fn deriv_and_integral_laws() = todo

@test
fn inv_log_exp_laws() = todo

@test
fn pow_and_sqrt_laws() = todo

@test
fn lagrange_and_interpolate_laws() = todo
