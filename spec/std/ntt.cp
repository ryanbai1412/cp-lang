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

/// In-place number-theoretic transform of length `2^k` modulo an NTT-friendly prime `P`.
/// Set `inverse: true` for the inverse transform (includes division by `n`).
/// Set `bit_reversed: true` to stay in DIF/DIT bit-reversed frequency order (skips bit-reversal). O(n log n).
pub fn ntt<P>(mut a: List<Mod<P>>, inverse: Bool = false, bit_reversed: Bool = false) = todo

/// Which arguments of a convolution over a semiring other than `(+, *)` have the shape that makes
/// it fast: convex for `min`, concave for `max`. `Neither` asks for the O(n m) loop by name.
pub enum Curved = Both | Right | Neither

/// The index op of a subset convolution: `i | j` over disjoint pairs `i & j == 0`. A value with a
/// transform impl, so user libraries can define more index ops the same way.
pub struct Disjoint

/// `on: subset` in `convolve`.
pub const subset = Disjoint()

/// `c[on(i, j)] = over.add` of `over.mul(a[i], b[j])`. The default is the polynomial product,
/// O((n + m) log(n + m)). `on: ^`, `|`, `&` use the bitwise transforms (lengths equal powers of
/// two, O(n log n)); `on: subset` is O(k^2 2^k); `on: gcd` and `on: lcm` index from 1 and cost
/// O(n log log n). Any `over:` but `Semiring(+, *)` needs `shape:` (`Both` merges slopes in
/// O(n + m), `Right` runs `monotone_minima` in O((n + m) log n), `Neither` is the O(n m) loop);
/// leaving it out is an error, not a hidden O(n m). With `cyclic: true`, `a.len == b.len` and
/// indices wrap modulo it.
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

/// O(k * 2^k). In-place subset-sum (`over: Sub`, default) or superset-sum (`over: Sup`) zeta transform
/// (`inverse: false`) or Mobius inverse (`inverse: true`) on any `IndexedMut` of power-of-two length `2^k`.
pub fn sos<S: IndexedMut>(mut a: S, over: SosOver = Sub, inverse: Bool = false) = todo

/// O(n log n). In-place Walsh-Hadamard (XOR) transform (`inverse: false` or `inverse: true`)
/// on any `IndexedMut` of power-of-two length `2^k` (`List<T>`, `[D]T`, mutable slices).
pub fn fwht<S: IndexedMut, T>(mut a: S, inverse: Bool = false) = todo

/// O(log^2 n) amortized per step, O(n log^2 n) total. Online (relaxed) NTT convolution:
/// supply `(a[k], b[k])` one step at a time and immediately receive `c[k] = sum_{i=0..=k} a[i] * b[k - i]`.
pub struct RelaxedConvolve<P>:
    a: List<Mod<P>>
    b: List<Mod<P>>
    c: List<Mod<P>>

    /// O(1). Start an empty online convolution.
    pub fn new() -> RelaxedConvolve<P> = todo

    /// O(log^2 k) amortized. Append `ai, bi` at index `k` and return `c[k]`.
    pub fn push(mut self, ai: Mod<P>, bi: Mod<P>) -> Mod<P> = todo

/// O(log^2 n) amortized per step. Semi-relaxed NTT convolution against a fixed `b`
/// (supply `a[k]` online and immediately receive `c[k] = sum_{i=0..=k} a[i] * b[k - i]`).
pub struct SemiRelaxedConvolve<P>:
    b: List<Mod<P>>
    a: List<Mod<P>>
    c: List<Mod<P>>

    /// O(b.len). Start an online convolution against fixed polynomial `b`.
    pub fn new(b: List<Mod<P>>) -> SemiRelaxedConvolve<P> = todo

    /// O(log^2 k) amortized. Append `ai` at index `k` and return `c[k]`.
    pub fn push(mut self, ai: Mod<P>) -> Mod<P> = todo

/// O(S log^2 S) for total degree `S`. Product of all polynomials in `polys` (returns `[1]` if empty).
pub fn convolve_all<P>(polys: List<List<Mod<P>>>) -> List<Mod<P>> = todo

/// O(k^2 * 2^k) for `n = 2^k`. Set power series exponential (`a[0] == 0`).
pub fn subset_exp<P>(a: List<Mod<P>>) -> List<Mod<P>> = todo

/// O(k^2 * 2^k) for `n = 2^k`. Set power series logarithm (`a[0] == 1`).
pub fn subset_log<P>(a: List<Mod<P>>) -> List<Mod<P>> = todo

@test
fn ntt_round_trip_and_convolve() = todo

@test
fn convolve_matches_brute_force_for_every_value_type() = todo

@test
fn sos_and_bitwise_convolutions() = todo
