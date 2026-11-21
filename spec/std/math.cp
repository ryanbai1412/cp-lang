//! Number theory on plain `Int`s.

/// The greatest common divisor, never negative; `gcd(0, 0) == 0`. O(log min(a, b)).
pub fn gcd(a: Int, b: Int) -> Int = todo

/// The least common multiple, never negative; stops on overflow.
pub fn lcm(a: Int, b: Int) -> Int = todo

/// O(1). Exact integer floor square root `floor(sqrt(x))` for `x >= 0` (no float rounding loss).
pub fn isqrt(x: Int) -> Int = todo

/// O(1). Exact integer ceiling square root `ceil(sqrt(x))` for `x >= 0`.
pub fn ceil_sqrt(x: Int) -> Int = todo

/// O(log k + log x). Exact integer `floor(x^(1/k))` for `x >= 0, k >= 1`.
pub fn nth_root(x: Int, k: Int) -> Int = todo

/// `num / den` with exactly `k` digits after the point, as text, from
/// integers alone: no float rounding. Halves round away from zero, and a
/// result that rounds to zero has no sign: `fixed(-1, 300, 2) == "0.00"`.
/// `fixed(t, 2, 1)` prints a half-integer. `|num| * 10^k * 2` must fit an `Int`.
pub fn fixed(num: Int, den: Int, k: Int) -> Str = todo

/// The number of ways to pick `k` of `n` things, exactly; 0 when `k` is outside `0..=n`.
/// O(min(k, n - k)); stops on overflow. For counts modulo a prime use `Comb` in `std.mod`.
pub fn choose(n: Int, k: Int) -> Int = todo

/// `n * (n - 1) / 2` for `n >= 2`, else `0`. O(1).
pub fn choose2(n: Int) -> Int = todo

/// `(g, x, y)` with `a * x + b * y == g == gcd(a, b)`. O(log min(a, b)).
pub fn ext_gcd(a: Int, b: Int) -> (g: Int, x: Int, y: Int) = todo

/// `ceil(a / b)` for any non-zero `b`. Built-in `/` is Euclidean division
/// (remainder is always non-negative), which already equals `floor(a / b)` for `b > 0`.
pub fn ceil_div(a: Int, b: Int) -> Int = todo

/// Euclidean quotient and remainder `(self / d, self % d)` (`0 <= r < |d|`,
/// `self == q * d + r`; traps on `d == 0`). Works as both `n.divmod(d)` and `divmod(n, d)`. O(1).
pub fn divmod(self: Int, d: Int) -> (Int, Int) = todo

/// The smallest prime factor of every number up to `n` (0 for 0 and 1), by a linear sieve.
/// O(n) time and memory.
pub struct Sieve:
    spf: List<0..=2_000_000_000>
    primes: List<0..=2_000_000_000>

    /// Linear sieve up to `n`, populating `spf` and `primes`. O(n) time and space.
    pub fn new(n: Int) -> Sieve = todo

    /// Whether `x` is prime (for `0 <= x <= n`). O(1).
    pub fn is_prime(self, x: Int) -> Bool = todo

    /// The prime factorization of `x >= 1` as `(prime, exponent)` pairs, smallest prime first. O(log x).
    pub fn factor(self, x: Int) -> List<(Int, Int)> = todo

    /// Euler's totient of `1 <= x <= n`. O(log x).
    pub fn phi(self, x: Int) -> Int = todo

    /// The Mobius function of `1 <= x <= n`: 0 if a square divides `x`, else -1 to the
    /// number of prime factors. O(log x).
    pub fn mobius(self, x: Int) -> Int = todo

    /// Table of Euler's totient `phi[0..=n]`. O(n).
    pub fn phi_table(self) -> List<Int> = todo

    /// Table of the Mobius function `mu[0..=n]` (`mu[0] == 0`, `mu[1] == 1`). O(n).
    pub fn mobius_table(self) -> List<Int> = todo

    /// Table of divisor counts `d[0..=n]`. O(n).
    pub fn div_count_table(self) -> List<Int> = todo

    /// Table of divisor sums `sigma[0..=n]`. O(n).
    pub fn div_sum_table(self) -> List<Int> = todo

    /// O(d(x) log d(x)). Sorted positive divisors of `1 <= x <= n`.
    pub fn divisors(self, x: Int) -> List<Int> = todo

    /// O(d(x) + log x). Generator yielding each positive divisor of `1 <= x <= n`.
    pub fn each_divisor(self, x: Int) -> yields Int = todo

/// Tabulate any multiplicative function `f` on `1..=n` in O(n) via a linear sieve.
/// `f(1) = T.one`, `f(a * b) = f(a) * f(b)` for coprime `a, b`, and
/// `f(p^e) = prime_pow(p, e, p^e)`. Index 0 holds `T.one`.
pub fn sieve_multiplicative<T>(n: Int, prime_pow: (Int, Int, Int) -> T) -> List<T> = todo

/// The number of bits needed to write `x >= 0` (`bit_len(0) == 0`, `bit_len(5) == 3`).
pub fn bit_len(x: Int) -> Int = todo

/// The digits of `x >= 0` in `base`, lowest first: `1203.digits()` is `[3, 0, 2, 1]`, and `0` has the one digit `0`.
pub fn digits(self: Int, base: Int = 10) -> List<Int> = todo

/// Which side of zero a number is on. `Int(s)` is -1, 0 or 1, so `x == Int(x.sign) * abs(x)`;
/// `-s` flips it and `max(s, .zero)` clamps it.
pub enum Sign: neg = -1, zero = 0, pos = 1

/// The side of zero `x` is on: `(b - a).sign`. O(1).
pub @get fn sign(self: Int) -> Sign = todo

/// Whether bit `i` of `x` is set: `x.bit(i)`, bit 0 the lowest. No bit from 63 up is set in an `Int` that is not negative.
pub fn bit(self: Int, i: Int) -> Bool = todo

/// Is `n` a power of two? `8.is_pow2` is true; `0`, `6` and negatives are not.
pub @get fn is_pow2(self: Int) -> Bool = todo

/// SplitMix64: mixes the 64 bits of `x` so that inputs close together land far apart. The
/// usual guard for `HashMap` keys and the step of a seeded generator; `U64` arithmetic wraps.
pub fn splitmix64(x: U64) -> U64 = todo

/// The prime factorization of `x >= 1` as `(prime, exponent)` pairs, smallest prime first,
/// for any 64-bit `Int`: trial division for `x <= 1_000_000` (O(sqrt x)), Pollard's rho +
/// deterministic Miller-Rabin above (O(x^(1/4) log x) expected).
pub fn factor(x: Int) -> List<(Int, Int)> = todo

/// The divisors of `x >= 1`, sorted. O(sqrt x).
pub fn divisors(x: Int) -> List<Int> = todo

/// All positive divisors of the integer with prime factorization `factors`, sorted. O(d log d).
pub fn divisors_from_factors(factors: List<(Int, Int)>) -> List<Int> = todo

fn ext_gcd_x(a: Int, b: Int) -> (Int, Int) = todo

/// The modular inverse of `a` modulo `m >= 1`, in `0..m`, or `none` if `gcd(a, m) != 1`. O(log m).
pub fn inv_mod(a: Int, m: Int) -> ?Int = todo

/// The solution of `x = r1 (mod m1)`, `x = r2 (mod m2)` as `(r, m)` with
/// `m = lcm(m1, m2)` and `0 <= r < m`, or `none` if there is none.
pub fn crt(r1: Int, m1: Int, r2: Int, m2: Int) -> ?(Int, Int) = todo

/// The solution of a system of congruences `x = r (mod m)` as `(r, m)` with `0 <= r < m`,
/// or `none` if inconsistent. Empty input yields `(0, 1)`. O(eqs.len * log lcm).
pub fn crt_all(eqs: List<(Int, Int)>) -> ?(Int, Int) = todo

/// `a * b % m` without overflow, rounded down to `0..m` for `m >= 1`. O(1), through `I128`
/// when the product could leave `Int`.
pub fn mul_mod(a: Int, b: Int, m: Int) -> Int = todo

/// `a ** e % m` for `e >= 0` and `1 <= m`. O(log e) multiplications, each O(1).
pub fn pow_mod(a: Int, e: Int, m: Int) -> Int = todo

/// Is `x` prime? Deterministic Miller-Rabin over the first twelve primes, exact for every
/// `Int`. O(log^2 x); `Sieve(n).is_prime(x)` is O(1) for many small `x`.
pub fn is_prime(x: Int) -> Bool = todo

/// `(y * y + c) % x` for `0 <= y < x` and `0 < c < x`, without leaving `Int`.
fn rho_step(y: Int, c: Int, x: Int) -> Int = todo

/// A divisor of the odd composite `x` other than 1 and `x`, by Pollard's rho with Brent's
/// batching. Expected O(x^(1/4) log x).
fn rho(x: Int) -> Int = todo

/// Euler's totient of `x >= 1`: how many of `1..=x` are coprime to `x`.
/// O(sqrt x) for `x <= 1_000_000`, O(x^(1/4) log x) expected above.
pub fn phi(x: Int) -> Int = todo

/// The smallest primitive root modulo the prime `p`: a `g` whose powers reach every
/// non-zero residue. O(p^(1/4) log p) expected to factor `p - 1`, then O(log^2 p) per try.
pub fn primitive_root(p: Int) -> Int = todo

/// The smallest non-negative `x` with `a^x = b (mod m)`, or `none` if none exists.
/// Handles arbitrary `m >= 1` including non-coprime `a` and `m`. O(sqrt m).
pub fn mod_log(a: Int, b: Int, m: Int) -> ?Int = todo

/// A square root `x` in `0..p` with `x * x = a (mod p)` for prime `p`, or `none`
/// if `a` is a quadratic non-residue. Tonelli-Shanks, O(log^2 p).
pub fn mod_sqrt(a: Int, p: Int) -> ?Int = todo

/// The sum of `(a * i + b) / m` over `i` in `0..n`, with floor division, for `n >= 0` and
/// `m >= 1`. O(log m), by the Euclid-like reduction.
pub fn floor_sum(n: Int, m: Int, a: Int, b: Int) -> Int = todo

/// O((hi - lo) log log hi + sqrt hi). Segmented sieve returning all primes in `lo..hi`.
pub fn primes_in(lo: Int, hi: Int) -> List<Int> = todo

/// O(n^(3/4) / log n), O(sqrt n) memory. Number of primes `<= n` (Meissel-Lehmer / Lucy_Hedgehog).
pub fn prime_count(n: Int) -> Int = todo

/// O(sqrt n). Generator yielding each distinct floor quotient `q = n / i` and its half-open range `lo..hi`.
pub fn floor_quotients(n: Int) -> yields (q: Int, lo: Int, hi: Int) = todo

/// O(x^(1/4) + d(x)) expected. Generator yielding each positive divisor of `x >= 1`.
pub fn each_divisor(x: Int) -> yields Int = todo

/// O(2^popcount(mask)). Generator yielding every submask of `mask` in descending order (including 0).
pub fn submasks(mask: Int) -> yields Int = todo

/// O(C(n, k)). Generator yielding all `k`-bit subsets of `0..n` in ascending order (Gosper's hack).
pub fn bit_subsets(n: Int, k: Int) -> yields Int = todo

/// O(n log n), O(n) memory. Dirichlet convolution `(a * b)[k] = sum_{d|k} a[d] * b[k/d]` on `1..=n`.
pub fn dirichlet_convolve<T>(a: List<T>, b: List<T>, n: Int) -> List<T> = todo

/// O(n log n), O(n) memory. Dirichlet inverse of `a` on `1..=n` (`a[1]` invertible).
pub fn dirichlet_inv<T>(a: List<T>, n: Int) -> List<T> = todo

/// O(n^(2/3)) time and memory. Stateful Du's sieve cache for prefix sums of a multiplicative function `f`
/// given prefix sums of `g` and `f * g`.
pub struct DirichletSum<T>:
    n: Int

    /// O(n^(2/3)). Build a prefix-sum cache for `f` up to `n` from prefix sums of `g` and `f * g`.
    pub fn new(n: Int, f_small: List<T>, g_sum: Int -> T, fg_sum: Int -> T) -> DirichletSum<T> = todo

    /// O(1) cached, O(n^(2/3)) total across all `n / i` queries. Prefix sum `sum_{k=1..=m} f(k)`.
    pub fn sum(mut self, m: Int) -> T = todo

/// O(n^(2/3)), O(n^(2/3)) memory. Sum of Euler's totient `sum_{k=1..=n} phi(k)`.
pub fn sum_phi(n: Int) -> Int = todo

/// O(n^(2/3)), O(n^(2/3)) memory. Mertens function `sum_{k=1..=n} mu(k)`.
pub fn sum_mobius(n: Int) -> Int = todo

/// O(n^(3/4) / log n), O(sqrt n) memory. Min_25 sieve: prefix sum `sum_{k=1..=n} f(k)` of a
/// multiplicative function `f` given its prime-power values `f(p, e, p^e)` and prime-sum polynomial
/// `prime_sum(n) = sum_{p <= n} f(p)` for each `floor(n / i)`.
pub fn sum_multiplicative<T>(n: Int, prime_pow: (Int, Int, Int) -> T, prime_sum: List<T>) -> T = todo

/// O(log min(|a|, |b|)). One solution `(x0, y0)` and steps `(step_x, step_y)` to `a*x + b*y = c`
/// (`x = x0 + k * step_x`, `y = y0 - k * step_y`), or `none` if `gcd(a, b)` does not divide `c`.
pub fn linear_diophantine(a: Int, b: Int, c: Int) -> ?(x0: Int, y0: Int, step_x: Int, step_y: Int) = todo

/// O(log m). Smallest `r in 0..step` and period `step` solving `a * x == b (mod m)`, or `none`.
pub fn linear_congruence(a: Int, b: Int, m: Int) -> ?(r: Int, step: Int) = todo

/// O(p^(1/4) + sqrt p) expected. Any `x` with `x^k == a (mod p)` for prime `p`, or `none`.
pub fn mod_kth_root(a: Int, k: Int, p: Int) -> ?Int = todo

/// O(m^(1/4) log m) expected. Multiplicative order of `a` modulo `m >= 1`, or `none` if `gcd(a, m) != 1`.
pub fn mult_order(a: Int, m: Int) -> ?Int = todo

/// O(log n). Jacobi symbol `(a / n)` in `{-1, 0, 1}` for odd `n > 0`.
pub fn jacobi(a: Int, n: Int) -> Int = todo

/// O(n^(1/4) log n) expected. Carmichael's lambda function `lambda(n)` for `n >= 1`.
pub fn carmichael(n: Int) -> Int = todo

/// O(log m). Smallest `x >= 0` with `lo <= (a * x) % m <= hi` (`0 <= lo <= hi < m`), or `none`.
pub fn min_mod_in_range(a: Int, m: Int, lo: Int, hi: Int) -> ?Int = todo

/// O(log min(|p|, |q|)). Continued fraction expansion `[a0, a1, ...]` of `p / q` (`q > 0`).
pub fn continued_fraction(p: Int, q: Int) -> List<Int> = todo

/// O(log max_den). Closest reduced fraction `p / q` (`1 <= q <= max_den`) to `num / den` via Stern-Brocot.
pub fn best_rational(num: Int, den: Int, max_den: Int) -> Ratio = todo

/// O(1) storage. Exact reduced rational `p / q` (`q > 0`, `gcd(|p|, q) == 1`).
pub struct Ratio(p: Int, q: Int):
    /// O(log min(|p|, |q|)). Reduced fraction `p / q`; traps on `q == 0`.
    pub fn new(p: Int, q: Int = 1) -> Ratio = todo

    /// O(1). Lift an `Int` to `Ratio(n, 1)`.
    pub fn from_int(n: Int) -> Ratio = todo

    /// O(1). Multiplicative inverse `q / p`; traps on `p == 0`.
    pub @get fn inv(self) -> Ratio = todo

    /// O(1). Floor `p / q` (Euclidean).
    pub @get fn floor(self) -> Int = todo

    /// O(1). Ceiling `ceil_div(p, q)`.
    pub @get fn ceil(self) -> Int = todo

    /// O(1). Absolute value `|p| / q`.
    pub @get fn abs(self) -> Ratio = todo

impl Add for Ratio:
    fn add(a, b) = todo

impl Sub for Ratio:
    fn sub(a, b) = todo

impl Neg for Ratio:
    fn neg(a) = todo

impl Mul for Ratio:
    fn mul(a, b) = todo

impl Div for Ratio:
    fn div(a, b) = todo

impl Ord for Ratio:
    fn lt(a, b) = todo

impl Show for Ratio:
    fn show(a) = todo

/// O(n) storage in base-10^9 limbs. Arbitrary-precision signed integer.
pub struct BigInt(neg: Bool, limbs: List<Int>):
    /// O(1). Construct a `BigInt` from an `Int`.
    pub fn from_int(x: Int) -> BigInt = todo

    /// O(s.len). Parse a decimal string (optional leading `-`).
    pub fn from_str(s: Str) -> BigInt = todo

    /// O(n). Decimal string representation.
    pub fn to_str(self) -> Str = todo

    /// O(n * m). Euclidean quotient and remainder `(self / d, self % d)`.
    pub fn divmod(self, d: BigInt) -> (BigInt, BigInt) = todo

    /// O(n). Absolute value.
    pub @get fn abs(self) -> BigInt = todo

    /// O(n^2). Greatest common divisor of `a` and `b`.
    pub fn gcd(a: BigInt, b: BigInt) -> BigInt = todo

impl Add for BigInt:
    fn add(a, b) = todo

impl Sub for BigInt:
    fn sub(a, b) = todo

impl Neg for BigInt:
    fn neg(a) = todo

impl Mul for BigInt:
    fn mul(a, b) = todo

impl Div for BigInt:
    fn div(a, b) = todo

impl Ord for BigInt:
    fn lt(a, b) = todo

impl Show for BigInt:
    fn show(a) = todo

@test
fn gcd_lcm_and_bezout() = todo

@test
fn choose_is_pascals_triangle() = todo

@test
fn division_helpers_round_the_right_way() = todo

@test
fn sieve_and_factorizations_agree() = todo

@test
fn crt_combines_or_refuses() = todo

@test
fn fixed_rounds_pads_and_drops_a_zero_sign() = todo

@test
fn bit_reads_one_bit() = todo

@test
fn digits_are_lowest_first() = todo

@test
fn sign_is_minus_one_zero_or_one() = todo

@test
fn is_pow2_is_one_set_bit() = todo

@test
fn splitmix64_matches_the_reference_values() = todo

@test
fn miller_rabin_agrees_with_the_sieve_and_refuses_pseudoprimes() = todo

@test
fn factor_agrees_with_sieve_and_handles_big_semiprimes() = todo

@test
fn pow_mod_phi_and_primitive_roots() = todo

@test
fn floor_sum_agrees_with_the_loop_for_every_sign() = todo

@test
fn choose2_inv_mod_crt_all_and_divisors_from_factors() = todo

@test
fn sieve_tables_and_multiplicative_agree() = todo

@test
fn mod_log_and_mod_sqrt_match_brute_force() = todo
