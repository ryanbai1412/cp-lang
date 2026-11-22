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

fn root_of(p: Int) -> Int = todo

fn mont_inv(p: Int) -> Int = todo

fn mred(x: Int, p: Int, pinv: Int) -> Int = todo

fn bit_reverse<T>(mut a: List<T>) = todo

fn ntt_raw(mut a: List<Int>, p: Int, g: Int, inverse: Bool) = todo

fn convolve_prime(a: List<Int>, b: List<Int>, p: Int, cyclic: Bool) -> List<Int> = todo

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
pub fn ntt<P>(mut a: List<Mod<P>>, inverse: Bool = false, bit_reversed: Bool = false) = todo

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
) -> List<T> = todo

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
pub fn sos<S: IndexedMut>(mut a: S, over: SosOver = Sub, inverse: Bool = false) = todo

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
pub fn fwht<S: IndexedMut, T>(mut a: S, inverse: Bool = false) = todo

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
    /// example: `type Z = Mod<17>; var rc = RelaxedConvolve<17>.new(); rc.push(Z(0), Z(0)) == Z(0)`
    pub fn new() -> RelaxedConvolve<P> = todo

    /// Append `a[k] = ai` and `b[k] = bi` at the next index `k` and return `c[k] = sum_{i=0..=k}
    /// a[i] * b[k - i]`.
    ///
    /// `c[k]` depends only on values already pushed, so `a[k+1]` and `b[k+1]` may be computed from
    /// it: the shape of online DP like `f[k] = sum f[i] g[k - i]`.
    ///
    /// cost: O(log^2 k) amortized
    /// traps: when `k` reaches the largest NTT length `P` supports
    /// example: `type Z = Mod<17>; var rc = RelaxedConvolve<17>.new(); rc.push(Z(1), Z(3)) == Z(3)`
    /// example: `type Z = Mod<17>; var rc = RelaxedConvolve<17>.new(); rc.push(Z(1), Z(3)); rc.push(Z(2), Z(4)) == Z(10)`
    pub fn push(mut self, ai: Mod<P>, bi: Mod<P>) -> Mod<P> = todo

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
    /// example: `type Z = Mod<17>; var sc = SemiRelaxedConvolve<17>.new([Z(1), Z(2)]); sc.push(Z(1)) == Z(1)`
    /// example: `type Z = Mod<17>; let e: List<Z> = []; var sc = SemiRelaxedConvolve<17>.new(e); sc.push(Z(5)) == Z(0)`
    pub fn new(b: List<Mod<P>>) -> SemiRelaxedConvolve<P> = todo

    /// Append `a[k] = ai` at the next index `k` and return `c[k] = sum_{i=0..=k} a[i] * b[k - i]`.
    ///
    /// cost: O(log^2 k) amortized
    /// traps: when `k` reaches the largest NTT length `P` supports
    /// example: `type Z = Mod<17>; var sc = SemiRelaxedConvolve<17>.new([Z(1), Z(2)]); sc.push(Z(1)); sc.push(Z(2)); sc.push(Z(5)) == Z(9)`
    pub fn push(mut self, ai: Mod<P>) -> Mod<P> = todo

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
pub fn convolve_all<P>(polys: List<List<Mod<P>>>) -> List<Mod<P>> = todo

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
pub fn subset_exp<P>(a: List<Mod<P>>) -> List<Mod<P>> = todo

/// The set power series logarithm of `a`, length `2^k`, with `a[0] == 1`: the inverse of
/// `subset_exp`, so `subset_exp(subset_log(a)) == a`.
///
/// cost: O(k^2 2^k)
/// empty: traps (0 is not a power of two)
/// order: indexed by mask; a new `List`, keyed from 0
/// traps: when `a.len` is not a power of two, or `a[0] != 1`
/// example: `type Z = Mod<17>; subset_log([Z(1), Z(1), Z(1), Z(2)]) == [Z(0), Z(1), Z(1), Z(1)]`
/// example: `type Z = Mod<17>; subset_log([Z(1), Z(0)]) == [Z(0), Z(0)]`
pub fn subset_log<P>(a: List<Mod<P>>) -> List<Mod<P>> = todo

@test
fn ntt_round_trip_and_convolve() = todo

@test
fn convolve_matches_brute_force_for_every_value_type() = todo

@test
fn sos_and_bitwise_convolutions() = todo
