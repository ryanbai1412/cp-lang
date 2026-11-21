//! Number-theoretic transform, exact and modular convolution, SOS, and bitwise transforms.
//!
//! Polynomials are coefficient lists, lowest degree first (`p[k]` is the coefficient of `x^k`).
//! `convolve`, `convolve_mod`, and `convolve_int` take `cyclic: Bool = false`:
//! - linear (`cyclic: false`): length `a.len + b.len - 1` (or `[]` if either input is empty)
//! - cyclic (`cyclic: true`): requires `a.len == b.len`, length `n = a.len`, index `(i + j) % n`.
use std.math.{is_pow2, mul_mod, pow_mod, primitive_root}
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

/// Convolution of `a` and `b` over `Mod<P>`. O(n log n) via NTT (O(na * nb) when small).
/// With `cyclic: true`, requires `a.len == b.len` and wraps indices modulo `a.len`.
pub fn convolve<P>(a: List<Mod<P>>, b: List<Mod<P>>, cyclic: Bool = false) -> List<Mod<P>> = todo

/// Convolution of integer lists modulo an arbitrary modulus `m >= 1` via 3-prime NTT + Garner CRT.
/// Exact for `m <= 10^9 + 7` and `a.len + b.len <= 2^23`. O(n log n).
pub fn convolve_mod(a: List<Int>, b: List<Int>, m: Int, cyclic: Bool = false) -> List<Int> = todo

/// Exact integer convolution (supports negative values) via 3-prime NTT + Garner CRT.
/// Exact whenever every output coefficient fits in `Int`. O(n log n).
pub fn convolve_int(a: List<Int>, b: List<Int>, cyclic: Bool = false) -> List<Int> = todo

/// Direction of a sum-over-subsets (SOS) zeta/Mobius transform.
pub enum SosOver = Sub | Sup

/// O(k * 2^k). In-place subset-sum (`over: Sub`, default) or superset-sum (`over: Sup`) zeta transform
/// (`inverse: false`) or Mobius inverse (`inverse: true`) on any `IndexedMut` of power-of-two length `2^k`.
pub fn sos<S: IndexedMut>(mut a: S, over: SosOver = Sub, inverse: Bool = false) = todo

/// Bitwise OR convolution: `out[i | j] += a[i] * b[j]`. Lengths must be equal powers of two. O(n log n).
pub fn convolve_or<T>(a: List<T>, b: List<T>) -> List<T> = todo

/// Bitwise AND convolution: `out[i & j] += a[i] * b[j]`. Lengths must be equal powers of two. O(n log n).
pub fn convolve_and<T>(a: List<T>, b: List<T>) -> List<T> = todo

/// O(n log n). In-place Walsh-Hadamard (XOR) transform (`inverse: false` or `inverse: true`)
/// on any `IndexedMut` of power-of-two length `2^k` (`List<T>`, `[D]T`, mutable slices).
pub fn fwht<S: IndexedMut, T>(mut a: S, inverse: Bool = false) = todo

/// Bitwise XOR convolution: `out[i ^ j] += a[i] * b[j]`. Lengths must be equal powers of two.
/// Works on `Int`, `Mod<P>`, and `Float`. O(n log n).
pub fn convolve_xor<T>(a: List<T>, b: List<T>) -> List<T> = todo

/// Subset convolution: `out[i | j] += a[i] * b[j]` over disjoint pairs `i & j == 0`.
/// Lengths must be equal powers of two `2^k`. O(k^2 * 2^k).
pub fn convolve_subset<T>(a: List<T>, b: List<T>) -> List<T> = todo

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

/// O((n + m) log(n + m)). Floating-point FFT convolution (linear or cyclic).
pub fn convolve_real(a: List<F64>, b: List<F64>, cyclic: Bool = false) -> List<F64> = todo

/// O(n log log n). GCD convolution: `out[g] = sum_{gcd(i, j) == g} a[i] * b[j]` (1-indexed, index 0 unused).
pub fn convolve_gcd<T>(a: List<T>, b: List<T>) -> List<T> = todo

/// O(n log log n). LCM convolution: `out[l] = sum_{lcm(i, j) == l} a[i] * b[j]` (1-indexed, index 0 unused).
pub fn convolve_lcm<T>(a: List<T>, b: List<T>) -> List<T> = todo

/// O(k^2 * 2^k) for `n = 2^k`. Set power series exponential (`a[0] == 0`).
pub fn subset_exp<P>(a: List<Mod<P>>) -> List<Mod<P>> = todo

/// O(k^2 * 2^k) for `n = 2^k`. Set power series logarithm (`a[0] == 1`).
pub fn subset_log<P>(a: List<Mod<P>>) -> List<Mod<P>> = todo

@test
fn ntt_round_trip_and_convolve() = todo

@test
fn convolve_mod_and_int_match_brute_force() = todo

@test
fn sos_and_bitwise_convolutions() = todo
