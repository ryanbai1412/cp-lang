//! Number theory on plain `Int`s.

/// The greatest common divisor of `a` and `b`, never negative; `gcd(0, 0) == 0`.
///
/// Signs are ignored: `gcd(a, b) == gcd(|a|, |b|)`, and `gcd(a, 0) == |a|`.
///
/// cost: O(log min(|a|, |b|))
/// traps: when the result is `2^63` (`gcd(Int.MIN, 0)` or `gcd(Int.MIN, Int.MIN)`)
/// example: `gcd(12, 18) == 6`
/// example: `gcd(-4, 6) == 2`
/// example: `gcd(0, -5) == 5`
/// example: `gcd(0, 0) == 0`
pub fn gcd(a: Int, b: Int) -> Int:
    var (x, y) = (abs(a), abs(b))
    while y != 0: (x, y) = (y, x % y)
    x

/// The least common multiple of `a` and `b`, never negative; 0 when either is 0.
///
/// Computed as `|a| / gcd(a, b) * |b|`, so it traps only when the true result overflows.
///
/// cost: O(log min(|a|, |b|))
/// traps: when the result overflows `Int`
/// example: `lcm(4, 6) == 12`
/// example: `lcm(-4, 6) == 12`
/// example: `lcm(0, 5) == 0`
pub fn lcm(a: Int, b: Int) -> Int = if a == 0 or b == 0: 0 else: abs(a / gcd(a, b) * b)

/// The exact floor square root `floor(sqrt(x))` of `x >= 0`, with no float rounding loss.
///
/// cost: O(1)
/// traps: when `x < 0`
/// example: `isqrt(15) == 3`
/// example: `isqrt(16) == 4`
/// example: `isqrt(0) == 0`
/// example: `isqrt(9_223_372_036_854_775_807) == 3_037_000_499`
pub fn isqrt(x: Int) -> Int:
    assert x >= 0
    var r = floor(sqrt(F64(x)))
    while r > 0 and r > x / r: r -= 1
    while r + 1 <= x / (r + 1): r += 1
    r

/// The exact ceiling square root `ceil(sqrt(x))` of `x >= 0`.
///
/// cost: O(1)
/// traps: when `x < 0`
/// example: `ceil_sqrt(15) == 4`
/// example: `ceil_sqrt(16) == 4`
/// example: `ceil_sqrt(0) == 0`
pub fn ceil_sqrt(x: Int) -> Int:
    let r = isqrt(x)
    if r * r == x: r else: r + 1

/// The exact floor `k`-th root of `x >= 0`, `k >= 1`: `iroot(x, 3)` is `floor(cbrt(x))`.
///
/// The largest `r >= 0` with `r^k <= x`, found without floats or overflow.
///
/// cost: O(log x)
/// traps: when `x < 0` or `k < 1`
/// example: `iroot(26, 3) == 2`
/// example: `iroot(27, 3) == 3`
/// example: `iroot(5, 1) == 5`
/// example: `iroot(0, 5) == 0`
pub fn iroot(x: Int, k: Int) -> Int:
    assert x >= 0 and k >= 1
    if k == 1 or x <= 1: return x
    var r = floor(F64(x) ** (F64(1) / F64(k)))
    while r > 1 and not pow_at_most(r, k, x): r -= 1
    while pow_at_most(r + 1, k, x): r += 1
    r

fn pow_at_most(r: Int, k: Int, x: Int) -> Bool:
    var p = 1
    for _ in 0..k:
        if p > x / r: return false
        p *= r
    true

/// `num / den` with exactly `k` digits after the point, as text, from integers alone.
///
/// No float is involved. Halves round away from zero, and a result that rounds to zero has no
/// sign: `fixed(-1, 300, 2) == "0.00"`. With `k == 0` there is no point. A negative `den` flips
/// the sign. `fixed(t, 2, 1)` prints a half-integer. `|num| * 10^k * 2` must fit an `Int`.
///
/// cost: O(k + digits of the result)
/// traps: when `den == 0`, `k < 0`, or `|num| * 10^k * 2` overflows
/// example: `fixed(1, 3, 2) == "0.33"`
/// example: `fixed(2, 3, 2) == "0.67"`
/// example: `fixed(-1, 300, 2) == "0.00"`
/// example: `fixed(5, 2, 0) == "3"`
/// example: `fixed(-5, 2, 0) == "-3"`
/// example: `fixed(7, 2, 1) == "3.5"`
/// example: `fixed(1, -4, 2) == "-0.25"`
pub fn fixed(num: Int, den: Int, k: Int) -> Str:
    assert den != 0 and k >= 0
    let scale = 10 ** k
    let units = (2 * abs(num) * scale + abs(den)) / (2 * abs(den))
    var out = if (num < 0) != (den < 0) and units > 0: "-" else: ""
    out += "{units / scale}"
    if k == 0: return out
    let frac = "{units % scale}"
    out += "."
    for _ in frac.len..k: out += "0"
    out + frac

/// The number of ways to pick `k` of `n` things, exactly; 0 when `k` is outside `0..=n`.
///
/// A negative `n` gives 0. The product is built so every intermediate value divides exactly
/// and is at most the result times `min(k, n - k)`. For counts modulo a prime use `Comb` in
/// `std.mod`.
///
/// cost: O(min(k, n - k))
/// traps: when the result overflows `Int`
/// example: `choose(5, 2) == 10`
/// example: `choose(5, 6) == 0`
/// example: `choose(5, -1) == 0`
/// example: `choose(60, 30) == 118_264_581_564_861_424`
pub fn choose(n: Int, k: Int) -> Int:
    if k < 0 or k > n: return 0
    var r = 1
    for i in 0..min(k, n - k): r = r * (n - i) / (i + 1)
    r

/// The number of unordered pairs from `n` things: `n * (n - 1) / 2` for `n >= 2`, else 0.
///
/// cost: O(1)
/// traps: when the result overflows `Int`
/// example: `choose2(5) == 10`
/// example: `choose2(1) == 0`
/// example: `choose2(-3) == 0`
pub fn choose2(n: Int) -> Int = if n < 2: 0 else: n * (n - 1) / 2

/// `(g, x, y)` with `a * x + b * y == g == gcd(a, b)`.
///
/// The Bezout pair is the one the iterative extended Euclidean algorithm gives with Euclidean
/// division (`a, b = b, a - (a / b) * b` until `b == 0`), negated at the end if `g` came out
/// negative. So `|x| <= max(1, |b| / g)` and `|y| <= max(1, |a| / g)`, and `ext_gcd(a, 0)` is
/// `(|a|, sign of a or 1, 0)`.
///
/// cost: O(log min(|a|, |b|))
/// traps: when `g` is `2^63`
/// example: `ext_gcd(240, 46) == (2, -9, 47)`
/// example: `ext_gcd(46, 240) == (2, 47, -9)`
/// example: `ext_gcd(-4, 6) == (2, 1, 1)`
/// example: `ext_gcd(0, 0) == (0, 1, 0)`
pub fn ext_gcd(a: Int, b: Int) -> (g: Int, x: Int, y: Int):
    var (r0, r1) = (a, b)
    var (x0, x1) = (1, 0)
    var (y0, y1) = (0, 1)
    while r1 != 0:
        let q = r0 / r1
        (r0, r1) = (r1, r0 - q * r1)
        (x0, x1) = (x1, x0 - q * x1)
        (y0, y1) = (y1, y0 - q * y1)
    if r0 < 0: (g: -r0, x: -x0, y: -y0) else: (g: r0, x: x0, y: y0)

/// `ceil(self / b)` for any non-zero `b`; `n.ceil_div(k)` or `ceil_div(n, k)`.
///
/// Built-in `/` is Euclidean division (the remainder is never negative), which already equals
/// `floor(a / b)` for `b > 0`; `ceil_div` is the rounding-up counterpart for either sign of `b`.
///
/// cost: O(1)
/// traps: when `b == 0`, or on overflow (`ceil_div(Int.MIN, -1)`)
/// example: `ceil_div(7, 2) == 4`
/// example: `ceil_div(-7, 2) == -3`
/// example: `ceil_div(7, -2) == -3`
/// example: `ceil_div(6, 3) == 2`
pub fn ceil_div(self: Int, b: Int) -> Int = if b > 0: -((-self) / b) else: -(self / (-b))

/// The Euclidean quotient and remainder `(self / d, self % d)`: `0 <= r < |d|` and `self == q * d + r`.
///
/// Works as both `n.divmod(d)` and `divmod(n, d)`.
///
/// cost: O(1)
/// traps: when `d == 0`, or on overflow (`Int.MIN.divmod(-1)`)
/// example: `7.divmod(3) == (2, 1)`
/// example: `(-7).divmod(3) == (-3, 2)`
/// example: `7.divmod(-3) == (-2, 1)`
/// example: `(-7).divmod(-3) == (3, 2)`
pub fn divmod(self: Int, d: Int) -> (Int, Int) = (self / d, self % d)

/// The smallest prime factor of every number up to `n` (0 for 0 and 1), by a linear sieve.
/// O(n) time and memory.
pub struct Sieve:
    spf: List<0..=2_000_000_000>
    primes: List<0..=2_000_000_000>

    /// A linear sieve up to `n`: the smallest prime factor of every `0..=n` and the primes in
    /// `2..=n`, ascending.
    ///
    /// `spf[0]` and `spf[1]` are 0. Queries take any `x` in `0..=n` (`1..=n` where 0 is meaningless).
    ///
    /// cost: O(n) time and memory
    /// traps: when `n < 0` or `n > 2_000_000_000`
    /// example: `Sieve(10).is_prime(7) == true`
    /// example: `Sieve(0).is_prime(0) == false`
    pub fn new(n: Int) -> Sieve:
        var spf: List<0..=2_000_000_000> = List.filled(n + 1, 0)
        var primes: List<0..=2_000_000_000> = []
        unchecked:
            for i in 2..=n:
                if spf[i] == 0:
                    spf[i] = i
                    primes.push(i)
                let si = spf[i]
                for p in primes:
                    if p > si or i * p > n: break
                    spf[i * p] = p
        Sieve(spf, primes)

    /// Whether `x` is prime, for `0 <= x <= n`: `spf[x] == x` and `x >= 2`.
    ///
    /// cost: O(1)
    /// traps: when `x` is outside `0..=n`
    /// example: `Sieve(10).is_prime(7) == true`
    /// example: `Sieve(10).is_prime(9) == false`
    /// example: `Sieve(10).is_prime(1) == false`
    pub fn is_prime(self, x: Int) -> Bool = x >= 2 and spf[x] == x

    /// The prime factorization of `x` in `1..=n` as `(prime, exponent)` pairs, by repeated `spf`.
    ///
    /// cost: O(log x)
    /// order: smallest prime first; a new `List`, keyed from 0
    /// traps: when `x` is outside `1..=n`
    /// example: `Sieve(100).factor(60) == [(2, 2), (3, 1), (5, 1)]`
    /// example: `Sieve(100).factor(97) == [(97, 1)]`
    /// example: `Sieve(100).factor(1) == []`
    pub fn factor(self, x: Int) -> List<(Int, Int)>:
        var out = List<(Int, Int)>()
        var rest = x
        while rest > 1:
            let p = spf[rest]
            var e = 0
            while rest % p == 0:
                rest /= p
                e += 1
            out.push((p, e))
        out

    /// Euler's totient of `x` in `1..=n`: how many of `1..=x` are coprime to `x`.
    ///
    /// cost: O(log x)
    /// traps: when `x` is outside `1..=n`
    /// example: `Sieve(100).phi(36) == 12`
    /// example: `Sieve(100).phi(1) == 1`
    pub fn phi(self, x: Int) -> Int:
        var acc = x
        for (p, _) in self.factor(x): acc = acc / p * (p - 1)
        acc

    /// The Mobius function of `x` in `1..=n`: 0 if a square above 1 divides `x`, else -1 to the
    /// number of prime factors.
    ///
    /// cost: O(log x)
    /// traps: when `x` is outside `1..=n`
    /// example: `Sieve(100).mobius(30) == -1`
    /// example: `Sieve(100).mobius(12) == 0`
    /// example: `Sieve(100).mobius(6) == 1`
    /// example: `Sieve(100).mobius(1) == 1`
    pub fn mobius(self, x: Int) -> Int:
        let f = self.factor(x)
        if f.any(fe => fe.1 > 1): 0 elif f.len % 2 == 0: 1 else: -1

    /// Euler's totient of every `0..=n`, with `phi[0] == 0`.
    ///
    /// cost: O(n)
    /// order: `phi[k]` at key `k`, `k` in `0..=n`; a new `List`
    /// traps: never
    /// example: `Sieve(6).phi_table() == [0, 1, 1, 2, 2, 4, 2]`
    pub fn phi_table(self) -> List<Int>:
        let n = spf.len - 1
        var out = List.filled(n + 1, 0)
        unchecked:
            if n >= 1: out[1] = 1
            for i in 2..=n:
                let p = spf[i]
                let q = i / p
                out[i] = if spf[q] == p: out[q] * p else: out[q] * (p - 1)
        out

    /// The Mobius function of every `0..=n`, with `mu[0] == 0` and `mu[1] == 1`.
    ///
    /// cost: O(n)
    /// order: `mu[k]` at key `k`, `k` in `0..=n`; a new `List`
    /// traps: never
    /// example: `Sieve(6).mobius_table() == [0, 1, -1, -1, 0, -1, 1]`
    pub fn mobius_table(self) -> List<Int>:
        let n = spf.len - 1
        var out = List.filled(n + 1, 0)
        unchecked:
            if n >= 1: out[1] = 1
            for i in 2..=n:
                let p = spf[i]
                let q = i / p
                out[i] = if spf[q] == p: 0 else: -out[q]
        out

    /// The number of positive divisors of every `0..=n`, with `d[0] == 0`.
    ///
    /// cost: O(n)
    /// order: `d[k]` at key `k`, `k` in `0..=n`; a new `List`
    /// traps: never
    /// example: `Sieve(6).div_count_table() == [0, 1, 2, 2, 3, 2, 4]`
    pub fn div_count_table(self) -> List<Int>:
        let n = spf.len - 1
        var out = List.filled(n + 1, 0)
        var cnt = List.filled(n + 1, 0)
        if n >= 1: out[1] = 1
        for i in 2..=n:
            let p = spf[i]
            let q = i / p
            if spf[q] == p:
                cnt[i] = cnt[q] + 1
                out[i] = out[q] / (cnt[q] + 1) * (cnt[i] + 1)
            else:
                cnt[i] = 1
                out[i] = out[q] * 2
        out

    /// The sum of the positive divisors of every `0..=n`, with `sigma[0] == 0`.
    ///
    /// cost: O(n)
    /// order: `sigma[k]` at key `k`, `k` in `0..=n`; a new `List`
    /// traps: when a sum overflows `Int`
    /// example: `Sieve(6).div_sum_table() == [0, 1, 3, 4, 7, 6, 12]`
    pub fn div_sum_table(self) -> List<Int>:
        let n = spf.len - 1
        var out = List.filled(n + 1, 0)
        var pp_sum = List.filled(n + 1, 0)
        if n >= 1:
            out[1] = 1
            pp_sum[1] = 1
        for i in 2..=n:
            let p = spf[i]
            let q = i / p
            if spf[q] == p:
                pp_sum[i] = pp_sum[q] * p + 1
                out[i] = out[q] / pp_sum[q] * pp_sum[i]
            else:
                pp_sum[i] = p + 1
                out[i] = out[q] * (p + 1)
        out

    /// The positive divisors of `x` in `1..=n`, built from the factorization, with no allocation.
    ///
    /// They come in mixed-radix order over the factorization: the exponent of the smallest prime
    /// changes fastest. That order is fixed but not sorted; use `.sorted()` for ascending.
    ///
    /// cost: O(log x) to start, O(1) per divisor
    /// order: mixed-radix over the exponents, smallest prime fastest (12 gives 1, 2, 4, 3, 6, 12)
    /// traps: when `x` is outside `1..=n`
    /// example: `Sieve(12).divisors(12).collect() == [1, 2, 4, 3, 6, 12]`
    /// example: `Sieve(12).divisors(1).collect() == [1]`
    pub fn divisors(self, x: Int) -> yields Int:
        for d in divisors_in_radix_order(self.factor(x)): yield d

/// Tabulate a multiplicative function `f` on `0..=n` in O(n) with a linear sieve.
///
/// `f(1) = T.one`, `f(a * b) = f(a) * f(b)` for coprime `a, b`, and `f(p^e) = prime_pow(p, e,
/// p^e)` for every prime power `p^e <= n`. Index 0 holds `T.one`.
///
/// cost: O(n) time and memory, plus one `prime_pow` call per prime power
/// order: `f(k)` at key `k`, `k` in `0..=n`; a new `List`
/// traps: when `n < 0`
/// example: `sieve_multiplicative(6, (p, e, pe) => e + 1) == [1, 1, 2, 2, 3, 2, 4]`
/// example: `sieve_multiplicative(4, (p, e, pe) => pe - pe / p) == [1, 1, 1, 2, 2]`
pub fn sieve_multiplicative<T>(n: Int, prime_pow: (Int, Int, Int) -> T) -> List<T>:
    assert n >= 0
    var res = List.filled(n + 1, T.one)
    var spf = List.filled(n + 1, 0)
    var primes = List<Int>()
    for i in 2..=n:
        if spf[i] == 0:
            primes.push(i)
            var (e, pe) = (1, i)
            while true:
                spf[pe] = i
                res[pe] = prime_pow(i, e, pe)
                if pe > n / i: break
                e += 1
                pe *= i
        for p in primes:
            var m = i * p
            if p == spf[i] or m > n: break
            var pe = p
            while true:
                spf[m] = p
                res[m] = res[i] * res[pe]
                m *= p
                if m > n: break
                pe *= p
    res

/// The number of bits needed to write `x >= 0`: `bit_len(0) == 0`, `bit_len(5) == 3`.
///
/// cost: O(1)
/// traps: when `x < 0`
/// example: `bit_len(0) == 0`
/// example: `bit_len(5) == 3`
/// example: `bit_len(8) == 4`
pub fn bit_len(x: Int) -> Int = bit_width(x)

/// The digits of `x >= 0` in `base`, lowest first; `0` has the one digit `0`.
///
/// cost: O(log_base x)
/// order: lowest digit first; a new `List`, keyed from 0
/// traps: when `self < 0` or `base < 2`
/// example: `1203.digits() == [3, 0, 2, 1]`
/// example: `0.digits() == [0]`
/// example: `10.digits(base: 2) == [0, 1, 0, 1]`
pub fn digits(self: Int, base: Int = 10) -> List<Int>:
    assert self >= 0 and base >= 2
    var (out, x) = ([self % base], self / base)
    while x > 0:
        out.push(x % base)
        x /= base
    out

/// `self` written in `base` (2..=36), highest digit first, lowercase letters past 9, a leading
/// `-` when negative. The inverse of `Str.parse_int`.
///
/// cost: O(log |self|)
/// traps: when `base` is outside `2..=36`
/// example: `255.to_str(base: 16) == "ff"`
/// example: `(-10).to_str(base: 2) == "-1010"`
/// example: `0.to_str() == "0"`
pub fn to_str(self: Int, base: Int = 10) -> Str:
    assert base >= 2 and base <= 36
    if self == 0: return "0"
    var cs: List<Char> = []
    var x = self
    while x != 0:
        let r = x % base
        let d = if x < 0 and r != 0: base - r else: r
        cs.push(if d < 10: Char(48 + d) else: Char(87 + d))
        x = if x < 0 and r != 0: x / base + 1 else: x / base
    if self < 0: cs.push('-')
    Str([cs[cs.len - 1 - i] for i in 0..cs.len])

/// Which side of zero a number is on. `Int(s)` is -1, 0 or 1, so `x == Int(x.sign) * abs(x)`;
/// `-s` flips it and `max(s, Sign.Zero)` clamps it.
pub enum Sign: Neg = -1, Zero = 0, Pos = 1

/// The side of zero `self` is on: `Neg`, `Zero` or `Pos`. `(b - a).sign` compares two values.
///
/// cost: O(1)
/// traps: never
/// example: `(-5).sign == Sign.Neg`
/// example: `0.sign == Sign.Zero`
/// example: `Int(7.sign) == 1`
@get pub fn sign(self: Int) -> Sign = if self > 0: Sign.Pos elif self < 0: Sign.Neg else: Sign.Zero

/// Whether bit `i` of `self` is set, bit 0 the lowest, in two's complement.
///
/// Bits from 63 up repeat the sign: None is set in a non-negative `Int` and all are set in a
/// negative one.
///
/// cost: O(1)
/// traps: when `i < 0`
/// example: `5.bit(0) == true`
/// example: `5.bit(1) == false`
/// example: `(-1).bit(100) == true`
/// example: `5.bit(70) == false`
pub fn bit(self: Int, i: Int) -> Bool = i < 64 and (self >> i) & 1 == 1

/// Is `self` a power of two? `self > 0 and self & (self - 1) == 0`.
///
/// cost: O(1)
/// traps: never
/// example: `8.is_pow2 == true`
/// example: `1.is_pow2 == true`
/// example: `6.is_pow2 == false`
/// example: `0.is_pow2 == false`
@get pub fn is_pow2(self: Int) -> Bool = self > 0 and self & (self - 1) == 0

/// SplitMix64: mixes the 64 bits of `x` so that inputs close together land far apart.
///
/// One full splitmix64 step on state `x`: `z = x + 0x9E3779B97F4A7C15`, then `z ^= z >> 30; z *=
/// 0xBF58476D1CE4E5B9; z ^= z >> 27; z *= 0x94D049BB133111EB; z ^= z >> 31`, all wrapping. It is
/// the usual guard for `HashMap` keys, and `std.rand.Rng.next` is this applied to its state.
///
/// cost: O(1)
/// traps: never
/// example: `splitmix64(0) == 0xE220A8397B1DCDAF`
/// example: `splitmix64(1) != splitmix64(2)`
pub fn splitmix64(x: U64) -> U64:
    var z: U64 = x + 0x9e3779b97f4a7c15
    z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9
    z = (z ^ (z >> 27)) * 0x94d049bb133111eb
    z ^ (z >> 31)

/// The prime factorization of `x >= 1` as `(prime, exponent)` pairs, for any 64-bit `Int`.
///
/// Trial division for `x <= 1_000_000`; Pollard's rho with deterministic Miller-Rabin above.
/// `factor(1)` is `[]`.
///
/// cost: O(sqrt x) for `x <= 1_000_000`; O(x^(1/4) log x) expected above
/// order: smallest prime first; a new `List`, keyed from 0
/// traps: when `x < 1`
/// example: `factor(60) == [(2, 2), (3, 1), (5, 1)]`
/// example: `factor(1) == []`
/// example: `factor(600_851_475_143) == [(71, 1), (839, 1), (1471, 1), (6857, 1)]`
/// example: `factor(1_000_000_007) == [(1_000_000_007, 1)]`
pub fn factor(x: Int) -> List<(Int, Int)>:
    assert x >= 1
    if x <= 1_000_000:
        var out = List<(Int, Int)>()
        var (rest, p) = (x, 2)
        while p * p <= rest:
            if rest % p == 0:
                var e = 0
                while rest % p == 0:
                    rest /= p
                    e += 1
                out.push((p, e))
            p += 1
        if rest > 1: out.push((rest, 1))
        return out
    var primes = List<Int>()
    var rest = x
    for p in 2..100:
        while rest % p == 0:
            primes.push(p)
            rest /= p
    var todo = [rest]
    while let m = todo.pop():
        if m == 1: continue
        if is_prime(m):
            primes.push(m)
        else:
            let d = rho(m)
            todo.push(d)
            todo.push(m / d)
    sort(mut primes)
    var out = List<(Int, Int)>()
    for p in primes:
        if not out.is_empty and out.last!.0 == p: out.last! = (p, out.last!.1 + 1)
        else: out.push((p, 1))
    out

/// The positive divisors of `x >= 1`, ascending.
///
/// cost: O(sqrt x)
/// order: ascending; a new `List`, keyed from 0
/// traps: when `x < 1`
/// example: `divisors(12) == [1, 2, 3, 4, 6, 12]`
/// example: `divisors(1) == [1]`
pub fn divisors(x: Int) -> List<Int>:
    var (small, large) = (List<Int>(), List<Int>())
    var d = 1
    while d * d <= x:
        if x % d == 0:
            small.push(d)
            if d * d != x: large.push(x / d)
        d += 1
    small + large.rev

/// Every positive divisor of the number whose factorization is `factors`, ascending.
///
/// `factors` is taken as `(prime, exponent)` pairs with distinct primes, as `factor` returns
/// them; it is not checked. The empty factorization is that of 1.
///
/// cost: O(d log d) for `d` divisors
/// empty: `[1]`
/// order: ascending; a new `List`, keyed from 0
/// traps: when an exponent is negative, or a divisor overflows `Int`
/// example: `divisors_from_factors([(2, 2), (3, 1)]) == [1, 2, 3, 4, 6, 12]`
/// example: `divisors_from_factors([]) == [1]`
pub fn divisors_from_factors(factors: List<(Int, Int)>) -> List<Int>:
    var out = [1]
    for (p, e) in factors:
        assert e >= 0, "a prime's exponent is at least 0"
        let cur = out.len
        var pk = 1
        for _ in 0..e:
            pk *= p
            for i in 0..cur: out.push(out[i] * pk)
    sort(mut out)
    out

fn ext_gcd_x(a: Int, b: Int) -> (Int, Int):
    var (r0, r1, x0, x1) = (a, b, 1, 0)
    while r1 != 0:
        let q = r0 / r1
        (r0, r1) = (r1, r0 - q * r1)
        (x0, x1) = (x1, x0 - q * x1)
    if r0 < 0: (-r0, -x0) else: (r0, x0)

/// The modular inverse of `a` modulo `m >= 1`, in `0..m`, or `None` if `gcd(a, m) != 1`.
///
/// `a` may be negative or at least `m`; it is reduced first. Modulo 1 the inverse is 0.
///
/// cost: O(log m)
/// traps: when `m < 1`
/// example: `inv_mod(3, 7) == Some(5)`
/// example: `inv_mod(-3, 7) == Some(2)`
/// example: `inv_mod(2, 4) == None`
/// example: `inv_mod(5, 1) == Some(0)`
pub fn inv_mod(a: Int, m: Int) -> ?Int:
    assert m >= 1
    let (g, x) = ext_gcd_x(a % m, m)
    if g != 1: return None
    x % m

/// The solution of `x = r1 (mod m1)`, `x = r2 (mod m2)` as `(r, m)` with `m = lcm(m1, m2)` and
/// `0 <= r < m`, or `None` if there is None.
///
/// The moduli need not be coprime; the residues may be any `Int`.
///
/// cost: O(log min(m1, m2))
/// traps: when `m1 < 1` or `m2 < 1`, or `lcm(m1, m2)` overflows
/// example: `crt(2, 3, 3, 5) == Some((8, 15))`
/// example: `crt(1, 4, 3, 6) == Some((9, 12))`
/// example: `crt(0, 2, 1, 4) == None`
pub fn crt(r1: Int, m1: Int, r2: Int, m2: Int) -> ?(Int, Int):
    let (g, x) = ext_gcd_x(m1, m2)
    if (r2 - r1) % g != 0: return None
    let m = m1 / g * m2
    let step = m2 / g
    // x = r1 + m1 * k with k = (r2 - r1) / g * inv(m1 / g) mod (m2 / g)
    let k = mul_mod((r2 - r1) / g % step, x % step, step)
    ((r1 + m1 * k) % m, m)

/// The solution of the system `x = r (mod m)` for every `(r, m)` in `eqs`, as `(r, m)` with
/// `0 <= r < m`, or `None` if inconsistent: `crt` folded left to right from `(0, 1)`.
///
/// cost: O(eqs.len * log lcm)
/// empty: `Some((0, 1))`
/// traps: when some `m < 1`, or the running lcm overflows
/// example: `crt_all([(2, 3), (3, 5), (2, 7)]) == Some((23, 105))`
/// example: `crt_all([(0, 2), (1, 2)]) == None`
/// example: `crt_all([]) == Some((0, 1))`
pub fn crt_all(eqs: List<(Int, Int)>) -> ?(Int, Int):
    var (r, m) = (0, 1)
    for (ri, mi) in eqs:
        assert mi >= 1, "every modulus is at least 1"
        let next = crt(r, m, ri, mi)
        if next == None: return None
        (r, m) = next!
    (r, m)

/// `a * b mod m` without overflow, in `0..m` for `m >= 1`: through `I128` when the product
/// could leave `Int`.
///
/// Negative operands are fine; the result is the Euclidean remainder.
///
/// cost: O(1)
/// traps: when `m < 1`
/// example: `mul_mod(1_000_000_000_000_000_000, 1_000_000_000_000_000_000, 1_000_000_007) == 2401`
/// example: `mul_mod(-3, 5, 7) == 6`
pub fn mul_mod(a: Int, b: Int, m: Int) -> Int:
    if -3_037_000_499 <= a <= 3_037_000_499 and -3_037_000_499 <= b <= 3_037_000_499: a * b % m
    else: Int(I128(a) * I128(b) % I128(m))

/// `a ** e mod m`, in `0..m`, for `e >= 0` and `m >= 1`; `a ** 0` is `1 % m`.
///
/// cost: O(log e) multiplications, each O(1)
/// traps: when `e < 0` or `m < 1`
/// example: `pow_mod(2, 10, 1000) == 24`
/// example: `pow_mod(-2, 3, 7) == 6`
/// example: `pow_mod(0, 0, 7) == 1`
/// example: `pow_mod(5, 0, 1) == 0`
pub fn pow_mod(a: Int, e: Int, m: Int) -> Int:
    var (acc, x, k) = (1 % m, a % m, e)
    while k > 0:
        if k % 2 == 1: acc = mul_mod(acc, x, m)
        x = mul_mod(x, x, m)
        k /= 2
    acc

/// Is `x` prime? Deterministic Miller-Rabin over the first twelve primes, exact for every `Int`.
///
/// Values below 2, negatives included, are not prime. `Sieve(n).is_prime(x)` is O(1) for many
/// small `x`.
///
/// cost: O(log^2 x)
/// traps: never
/// example: `is_prime(2) == true`
/// example: `is_prime(1_000_000_007) == true`
/// example: `is_prime(561) == false`
/// example: `is_prime(1) == false`
/// example: `is_prime(-7) == false`
pub fn is_prime(x: Int) -> Bool:
    if x < 2: return false
    let bases = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37]
    for p in bases:
        if x % p == 0: return x == p
    var (odd, twos) = (x - 1, 0)
    while odd % 2 == 0:
        odd /= 2
        twos += 1
    for a in bases:
        var y = pow_mod(a, odd, x)
        if y == 1 or y == x - 1: continue
        var composite = true
        for _ in 1..twos:
            y = mul_mod(y, y, x)
            if y == x - 1:
                composite = false
                break
        if composite: return false
    true

/// `(y * y + c) % x` for `0 <= y < x` and `0 < c < x`, without leaving `Int`.
fn rho_step(y: Int, c: Int, x: Int) -> Int:
    let s = mul_mod(y, y, x)
    if s >= x - c: s - (x - c) else: s + c

/// A divisor of the odd composite `x` other than 1 and `x`, by Pollard's rho with Brent's
/// batching. Expected O(x^(1/4) log x).
fn rho(x: Int) -> Int:
    var c = 1
    while true:
        var (y, q, g, r) = (2, 1, 1, 1)
        var (ys, z) = (2, 2)
        while g == 1:
            z = y
            for _ in 0..r: y = rho_step(y, c, x)
            var k = 0
            while k < r and g == 1:
                ys = y
                for _ in 0..min(128, r - k):
                    y = rho_step(y, c, x)
                    q = mul_mod(q, abs(z - y), x)
                g = gcd(q, x)
                k += 128
            r *= 2
        if g == x:
            g = 1
            while g == 1:
                ys = rho_step(ys, c, x)
                g = gcd(abs(z - ys), x)
        if g != x: return g
        c += 1
    x

/// Euler's totient of `x >= 1`: how many of `1..=x` are coprime to `x`.
///
/// cost: O(sqrt x) for `x <= 1_000_000`; O(x^(1/4) log x) expected above
/// traps: when `x < 1`
/// example: `phi(36) == 12`
/// example: `phi(1) == 1`
/// example: `phi(1_000_000_007) == 1_000_000_006`
pub fn phi(x: Int) -> Int:
    var acc = x
    for (p, _) in factor(x): acc = acc / p * (p - 1)
    acc

/// The smallest primitive root modulo the prime `p`: the least `g >= 1` whose powers reach every
/// non-zero residue.
///
/// cost: O(p^(1/4) log p) expected to factor `p - 1`, then O(log^2 p) per candidate
/// traps: when `p` is not prime
/// example: `primitive_root(7) == 3`
/// example: `primitive_root(998_244_353) == 3`
/// example: `primitive_root(2) == 1`
pub fn primitive_root(p: Int) -> Int:
    assert is_prime(p)
    if p == 2: return 1
    let parts = [q for (q, _) in factor(p - 1)]
    var g = 2
    while parts.any(q => pow_mod(g, (p - 1) / q, p) == 1): g += 1
    g

/// The least `x >= 0` with `a^x = b (mod m)`, or `None` if there is None.
///
/// Any `m >= 1` works, including `a` not coprime to `m` (the shared factor is peeled off before
/// baby-step giant-step). `a^0` is `1 % m`, and `a` and `b` are reduced first.
///
/// cost: O(sqrt m)
/// ties: the least `x`
/// traps: when `m < 1`
/// example: `mod_log(3, 6, 7) == Some(3)`
/// example: `mod_log(2, 3, 7) == None`
/// example: `mod_log(0, 0, 5) == Some(1)`
/// example: `mod_log(2, 1, 1) == Some(0)`
pub fn mod_log(a: Int, b: Int, m: Int) -> ?Int:
    assert m >= 1
    let (a0, b0) = (a % m, b % m)
    if m == 1 or b0 == 1: return 0
    var (k, add, cur_b, cur_m) = (1, 0, b0, m)
    while true:
        let g = gcd(a0, cur_m)
        if g == 1: break
        if k == cur_b: return add
        if cur_b % g != 0: return None
        cur_b /= g
        cur_m /= g
        k = mul_mod(k, a0 / g, cur_m)
        add += 1
    if k == cur_b: return add
    var n = 1
    while n * n < cur_m: n += 1
    var table = HashMap<Int, Int>()
    var rhs = cur_b
    for q in 0..=n:
        table[rhs] = q
        rhs = mul_mod(rhs, a0, cur_m)
    let an = pow_mod(a0, n, cur_m)
    var cur = k
    for p in 1..=n:
        cur = mul_mod(cur, an, cur_m)
        if let q = table.get(cur): return p * n - q + add
    None

/// A square root `x` in `0..p` with `x * x = a (mod p)` for prime `p`, or `None` if `a` is a
/// quadratic non-residue (Tonelli-Shanks).
///
/// Of the two roots `x` and `p - x` it returns the smaller; the root of 0 is 0. `a` is reduced
/// first.
///
/// cost: O(log^2 p)
/// ties: the smaller root
/// traps: when `p` is not prime
/// example: `mod_sqrt(2, 7) == Some(3)`
/// example: `mod_sqrt(9, 7) == Some(3)`
/// example: `mod_sqrt(3, 7) == None`
/// example: `mod_sqrt(0, 7) == Some(0)`
pub fn mod_sqrt(a: Int, p: Int) -> ?Int:
    assert is_prime(p)
    let x = a % p
    if x == 0 or p == 2: return x
    if pow_mod(x, (p - 1) / 2, p) != 1: return None
    if p % 4 == 3: return pow_mod(x, (p + 1) / 4, p)
    var (q, s) = (p - 1, 0)
    while q % 2 == 0:
        q /= 2
        s += 1
    var z = 2
    while pow_mod(z, (p - 1) / 2, p) != p - 1: z += 1
    var m = s
    var c = pow_mod(z, q, p)
    var t = pow_mod(x, q, p)
    var r = pow_mod(x, (q + 1) / 2, p)
    while t != 1:
        var (i, t2) = (1, mul_mod(t, t, p))
        while t2 != 1:
            t2 = mul_mod(t2, t2, p)
            i += 1
        let b = pow_mod(c, 1 << (m - i - 1), p)
        m = i
        c = mul_mod(b, b, p)
        t = mul_mod(t, c, p)
        r = mul_mod(r, b, p)
    r

/// The sum of `(a * i + b) / m` over `i` in `0..n`, with floor (Euclidean) division, by the
/// Euclid-like reduction. Any sign of `a` and `b` works.
///
/// cost: O(log m)
/// traps: when `n < 0` or `m < 1`, or the sum overflows
/// example: `floor_sum(4, 10, 6, 3) == 3`
/// example: `floor_sum(3, 2, -1, 0) == -2`
/// example: `floor_sum(0, 5, 1, 1) == 0`
pub fn floor_sum(n: Int, m: Int, a: Int, b: Int) -> Int:
    assert n >= 0 and m > 0
    var (count, den, slope, start) = (n, m, a, b)
    var total = 0
    while true:
        total += count * (count - 1) / 2 * (slope / den) + count * (start / den)
        slope %= den
        start %= den
        let top = slope * count + start
        if top < den: break
        count = top / den
        start = top % den
        swap(mut den, mut slope)
    total

/// Every prime in `lo..hi`, by a segmented sieve; `[]` when the range is empty or below 2.
///
/// cost: O((hi - lo) log log hi + sqrt hi)
/// order: ascending; a new `List`, keyed from 0
/// traps: never
/// example: `primes_in(10, 30) == [11, 13, 17, 19, 23, 29]`
/// example: `primes_in(0, 3) == [2]`
/// example: `primes_in(20, 10) == []`
pub fn primes_in(lo: Int, hi: Int) -> List<Int>:
    let a = max(lo, 2)
    if hi <= a: return []
    let r = isqrt(hi - 1)
    var small = List.filled(r + 1, true)
    var seg = List.filled(hi - a, true)
    for p in 2..=r:
        if not small[p]: continue
        var m = p * p
        while m <= r:
            small[m] = false
            m += p
        m = max(p * p, (a + p - 1) / p * p)
        while m < hi:
            seg[m - a] = false
            m += p
    [a + i for i in 0..hi - a if seg[i]]

/// The number of primes `<= n` (Lucy_Hedgehog); 0 for `n < 2`.
///
/// cost: O(n^(3/4) / log n) time, O(sqrt n) memory
/// traps: never
/// example: `prime_count(10) == 4`
/// example: `prime_count(100) == 25`
/// example: `prime_count(1) == 0`
pub fn prime_count(n: Int) -> Int:
    if n < 2: return 0
    let r = isqrt(n)
    var lo = [i - 1 for i in 0..=r]
    var hi = [if i == 0: 0 else: n / i - 1 for i in 0..=r]
    for p in 2..=r:
        if lo[p] == lo[p - 1]: continue
        let pc = lo[p - 1]
        let p2 = p * p
        for i in 1..=min(r, n / p2):
            let d = i * p
            hi[i] -= (if d <= r: hi[d] else: lo[n / d]) - pc
        var i = r
        while i >= p2:
            lo[i] -= lo[i / p] - pc
            i -= 1
    hi[1]

/// Each distinct floor quotient `q = n / i` for `i` in `from..=n`, with the half-open range
/// `lo..hi` of the `i` that give it.
///
/// The first range is clipped to start at `from`. Nothing is yielded when `n < from`.
///
/// cost: O(sqrt n)
/// empty: yields nothing when `n < from`
/// order: by `i`, increasing (so `q` decreasing)
/// traps: when `from < 1`
/// example: `floor_quotients(10).collect() == [(10, 1, 2), (5, 2, 3), (3, 3, 4), (2, 4, 6), (1, 6, 11)]`
/// example: `floor_quotients(10, from: 4).collect() == [(2, 4, 6), (1, 6, 11)]`
/// example: `floor_quotients(0).collect() == []`
pub fn floor_quotients(n: Int, from: Int = 1) -> yields (q: Int, lo: Int, hi: Int):
    var i = max(from, 1)
    while i <= n:
        let q = n / i
        let j = n / q + 1
        yield (q:, lo: i, hi: j)
        i = j

/// Each positive divisor of `x >= 1`, from `factor(x)`, without building a list.
///
/// cost: O(x^(1/4) + d(x)) expected
/// order: mixed-radix over the exponents, smallest prime fastest (12 gives 1, 2, 4, 3, 6, 12)
/// traps: when `x < 1`
/// example: `each_divisor(12).collect() == [1, 2, 4, 3, 6, 12]`
/// example: `each_divisor(1).collect() == [1]`
pub fn each_divisor(x: Int) -> yields Int:
    for d in divisors_in_radix_order(factor(x)): yield d

fn divisors_in_radix_order(fs: List<(Int, Int)>) -> yields Int:
    var e = List.filled(fs.len, 0)
    var pe = List.filled(fs.len, 1)
    var cur = 1
    yield cur
    while true:
        var i = 0
        while i < fs.len and e[i] == fs[i].1:
            cur /= pe[i]
            e[i] = 0
            pe[i] = 1
            i += 1
        if i == fs.len: break
        e[i] += 1
        pe[i] *= fs[i].0
        cur *= fs[i].0
        yield cur

/// Every submask of `mask`, in descending order, ending with 0.
///
/// cost: O(2^popcount(mask))
/// order: descending
/// traps: when `mask < 0`
/// example: `submasks(5).collect() == [5, 4, 1, 0]`
/// example: `submasks(0).collect() == [0]`
pub fn submasks(mask: Int) -> yields Int:
    var s = mask
    while true:
        yield s
        if s == 0: break
        s = (s - 1) & mask

/// Every subset of `0..n` with exactly `k` elements, as a bitmask, in ascending order (Gosper's
/// hack).
///
/// cost: O(C(n, k))
/// empty: yields nothing when `k > n`; yields just 0 when `k == 0`
/// order: ascending
/// traps: when `n < 0`, `k < 0` or `n > 62`
/// example: `bit_subsets(4, 2).collect() == [3, 5, 6, 9, 10, 12]`
/// example: `bit_subsets(3, 0).collect() == [0]`
/// example: `bit_subsets(2, 3).collect() == []`
pub fn bit_subsets(n: Int, k: Int) -> yields Int:
    if k == 0: yield 0
    if k >= 1 and k <= n:
        var s = (1 << k) - 1
        while s < 1 << n:
            yield s
            let c = s & -s
            let r = s + c
            s = (((r ^ s) >> 2) / c) | r

/// The Dirichlet convolution `(a * b)[k] = sum_{d | k} a[d] * b[k / d]` for `k` in `1..=n`.
///
/// `a` and `b` are indexed by the integer itself: index 0 is ignored and `c[0]` is `T.zero`.
///
/// cost: O(n log n) time, O(n) memory
/// empty: `[T.zero]` when `n == 0`
/// order: `c[k]` at key `k`, `k` in `0..=n`; a new `List`
/// traps: when `n < 0`, or `a.len` or `b.len` is below `n + 1`
/// example: `dirichlet_convolve([0, 1, 1, 1, 1, 1, 1], [0, 1, 1, 1, 1, 1, 1], 6) == [0, 1, 2, 2, 3, 2, 4]`
/// example: `dirichlet_convolve([0, 1, 2, 3], [0, 1, 0, 0], 3) == [0, 1, 2, 3]`
pub fn dirichlet_convolve<T>(a: List<T>, b: List<T>, n: Int) -> List<T> = todo

/// The Dirichlet inverse of `a` on `1..=n`: the `b` with `a * b` the unit (1 at 1, 0 elsewhere).
///
/// Indexed like `dirichlet_convolve`: index 0 is ignored and `b[0]` is `T.zero`. `a[1]` must be
/// invertible in `T`; for `Int` that means `a[1]` is 1 or -1.
///
/// cost: O(n log n) time, O(n) memory
/// empty: `[T.zero]` when `n == 0`
/// order: `b[k]` at key `k`, `k` in `0..=n`; a new `List`
/// traps: when `n < 0`, `a.len < n + 1`, or `a[1]` is not invertible
/// example: `dirichlet_inv([0, 1, 1, 1, 1, 1, 1], 6) == [0, 1, -1, -1, 0, -1, 1]`
pub fn dirichlet_inv<T>(a: List<T>, n: Int) -> List<T> = todo

/// O(n^(2/3)) time and memory. Stateful Du's sieve cache for prefix sums of a multiplicative function `f`
/// given prefix sums of `g` and `f * g`.
pub struct DirichletSum<T>:
    n: Int

    /// A Du's-sieve cache for the prefix sums `F(m) = sum_{k=1..=m} f(k)` of a multiplicative `f`, up
    /// to `n`.
    ///
    /// `f_small[k]` is `f(k)` for `k` in `1..f_small.len` (index 0 ignored); sums up to there come
    /// from its prefix sums. Larger `m` use `g_sum(m) = sum_{k<=m} g(k)` and `fg_sum(m) = sum_{k<=m}
    /// (f * g)(k)` for a Dirichlet partner `g` with `g(1)` invertible:
    /// `F(m) = (fg_sum(m) - sum_{i=2..=m} g(i) F(m / i)) / g(1)`, grouped by floor quotients and
    /// memoized. A table of about `n^(2/3)` entries gives the stated cost.
    ///
    /// cost: O(f_small.len) to build; queries as `sum`
    /// empty: an empty `f_small` makes every query use the recursion
    /// traps: when `n < 0`
    /// example: `var ds = DirichletSum<Int>(10, [0, 1, 1, 2, 2], m => m, m => m * (m + 1) / 2); ds.sum(10) == 32`
    pub fn new(n: Int, f_small: List<T>, g_sum: Int -> T, fg_sum: Int -> T) -> DirichletSum<T> = todo

    /// The prefix sum `F(m) = sum_{k=1..=m} f(k)`, with `F(0) == 0`.
    ///
    /// Results are memoized, so all the `n / i` values together cost O(n^(2/3)) when `f_small` has
    /// about `n^(2/3)` entries.
    ///
    /// cost: O(1) cached; O(n^(2/3)) total across all `n / i` queries
    /// traps: when `m` is outside `0..=n`, or `g(1)` is not invertible
    /// example: `var ds = DirichletSum<Int>(10, [0, 1, 1, 2, 2, 4, 2, 6, 4, 6, 4], m => m, m => m * (m + 1) / 2); ds.sum(10) == 32`
    /// example: `var ds = DirichletSum<Int>(10, [0, 1, 1, 2, 2], m => m, m => m * (m + 1) / 2); ds.sum(0) == 0`
    pub fn sum(mut self, m: Int) -> T = todo

/// The totient summatory function `sum_{k=1..=n} phi(k)`; 0 for `n <= 0`.
///
/// cost: O(n^(2/3)) time and memory
/// traps: when the sum overflows `Int`
/// example: `sum_phi(10) == 32`
/// example: `sum_phi(1) == 1`
/// example: `sum_phi(0) == 0`
pub fn sum_phi(n: Int) -> Int = todo

/// The Mertens function `sum_{k=1..=n} mu(k)`; 0 for `n <= 0`.
///
/// cost: O(n^(2/3)) time and memory
/// traps: never
/// example: `sum_mobius(10) == -1`
/// example: `sum_mobius(1) == 1`
/// example: `sum_mobius(0) == 0`
pub fn sum_mobius(n: Int) -> Int = todo

/// The prefix sum `sum_{k=1..=n} f(k)` of a multiplicative `f` (Min_25 sieve).
///
/// `f(p^e) = prime_pow(p, e, p^e)`. `prime_sum` holds `sum_{p <= v} f(p)` for every distinct
/// value `v` of `n / i`, `i` in `1..=n`, in ascending order of `v` (for `n = 10`: `v` = 1, 2, 3,
/// 5, 10).
///
/// cost: O(n^(3/4) / log n) time, O(sqrt n) memory
/// empty: `T.zero` when `n == 0` (with `prime_sum == []`)
/// order: `prime_sum` is read in ascending order of `v`
/// traps: when `n < 0`, or `prime_sum.len` is not the number of distinct `n / i`
/// example: `sum_multiplicative(10, (p, e, pe) => 1, [0, 1, 2, 3, 4]) == 10`
/// example: `sum_multiplicative(10, (p, e, pe) => e + 1, [0, 2, 4, 6, 8]) == 27`
pub fn sum_multiplicative<T>(n: Int, prime_pow: (Int, Int, Int) -> T, prime_sum: List<T>) -> T = todo

/// Every integer solution of `a*x + b*y = c` as `x = x0 + k * step_x`, `y = y0 - k * step_y`,
/// or `None` if `gcd(a, b)` does not divide `c`.
///
/// With `g = gcd(a, b)`: `step_x = b / g`, `step_y = a / g` (signs kept), and `x0` is the least
/// non-negative solution for `x` when `b != 0`, so the result is canonical. When `b == 0`,
/// `x0 = c / a` and `y0 = 0`.
///
/// cost: O(log min(|a|, |b|))
/// traps: when `a == b == 0` (the solutions are not one family), or `y0` overflows
/// example: `linear_diophantine(240, 46, 4) == Some((5, -26, 23, 120))`
/// example: `linear_diophantine(3, 0, 6) == Some((2, 0, 0, 1))`
/// example: `linear_diophantine(2, 4, 3) == None`
pub fn linear_diophantine(a: Int, b: Int, c: Int) -> ?(x0: Int, y0: Int, step_x: Int, step_y: Int):
    assert a != 0 or b != 0
    let (g, x, _) = ext_gcd(a, b)
    if c % g != 0: return None
    let (sx, sy) = (b / g, a / g)
    if b == 0: return Some((x0: c / a, y0: 0, step_x: sx, step_y: sy))
    let m = abs(sx)
    let x0 = mul_mod(((x % m) + m) % m, ((c / g) % m + m) % m, m)
    Some((x0:, y0: (c - a * x0) / b, step_x: sx, step_y: sy))

/// The solutions of `a * x = b (mod m)` as `x = r (mod step)`: the least `r` in `0..step` and
/// the period `step = m / gcd(a, m)`, or `None` if `gcd(a, m)` does not divide `b`.
///
/// `a` and `b` are reduced first.
///
/// cost: O(log m)
/// traps: when `m < 1`
/// example: `linear_congruence(6, 4, 10) == Some((4, 5))`
/// example: `linear_congruence(2, 1, 4) == None`
/// example: `linear_congruence(0, 0, 3) == Some((0, 1))`
pub fn linear_congruence(a: Int, b: Int, m: Int) -> ?(r: Int, step: Int):
    assert m >= 1
    let (a2, b2) = (a % m, b % m)
    let g = gcd(a2, m)
    if b2 % g != 0: return None
    let step = m / g
    if step == 1: return Some((r: 0, step:))
    Some((r: mul_mod(b2 / g, inv_mod(a2 / g, step)!, step), step:))

/// The least `x` in `0..p` with `x^k = a (mod p)` for prime `p` and `k >= 1`, or `None`.
///
/// `a` is reduced first; the root of 0 is 0.
///
/// cost: O(p^(1/4) + sqrt p) expected to find one root, plus O(gcd(k, p - 1)) to pick the least
/// ties: the least root
/// traps: when `p` is not prime or `k < 1`
/// example: `mod_kth_root(6, 3, 7) == Some(3)`
/// example: `mod_kth_root(2, 3, 7) == None`
/// example: `mod_kth_root(4, 2, 7) == Some(2)`
/// example: `mod_kth_root(0, 5, 7) == Some(0)`
pub fn mod_kth_root(a: Int, k: Int, p: Int) -> ?Int = todo

/// The multiplicative order of `a` modulo `m >= 1`: the least `k >= 1` with `a^k = 1 (mod m)`,
/// or `None` if `gcd(a, m) != 1`. Modulo 1 the order is 1.
///
/// cost: O(m^(1/4) log m) expected
/// traps: when `m < 1`
/// example: `mult_order(2, 7) == Some(3)`
/// example: `mult_order(3, 7) == Some(6)`
/// example: `mult_order(2, 4) == None`
/// example: `mult_order(5, 1) == Some(1)`
pub fn mult_order(a: Int, m: Int) -> ?Int:
    assert m >= 1
    if m == 1: return Some(1)
    let a2 = (a % m + m) % m
    if gcd(a2, m) != 1: return None
    var t = carmichael(m)
    for (p, _) in factor(t):
        while t % p == 0 and pow_mod(a2, t / p, m) == 1: t /= p
    Some(t)

/// The Jacobi symbol `(a / n)` in `{-1, 0, 1}` for odd `n > 0`; 0 exactly when `gcd(a, n) != 1`.
///
/// For prime `n` it is the Legendre symbol. `a` may be any `Int`; `(a / 1) == 1`.
///
/// cost: O(log n)
/// traps: when `n <= 0` or `n` is even
/// example: `jacobi(2, 7) == 1`
/// example: `jacobi(3, 7) == -1`
/// example: `jacobi(7, 7) == 0`
/// example: `jacobi(2, 15) == 1`
pub fn jacobi(a: Int, n: Int) -> Int:
    assert n > 0 and n % 2 == 1
    var (x, y) = (a % n, n)
    var res = 1
    while x != 0:
        while x % 2 == 0:
            x /= 2
            if y % 8 == 3 or y % 8 == 5: res = -res
        swap(mut x, mut y)
        if x % 4 == 3 and y % 4 == 3: res = -res
        x %= y
    if y == 1: res else: 0

/// Carmichael's function `lambda(n)` for `n >= 1`: the least `k >= 1` with `a^k = 1 (mod n)`
/// for every `a` coprime to `n`.
///
/// cost: O(n^(1/4) log n) expected
/// traps: when `n < 1`
/// example: `carmichael(1) == 1`
/// example: `carmichael(8) == 2`
/// example: `carmichael(15) == 4`
/// example: `carmichael(7) == 6`
pub fn carmichael(n: Int) -> Int:
    assert n >= 1
    var res = 1
    for (p, e) in factor(n):
        var pe1 = 1
        for _ in 1..e: pe1 *= p
        let l = if p == 2 and e >= 3: pe1 / 2 else: pe1 * (p - 1)
        res = lcm(res, l)
    res

/// The least `x >= 0` with `lo <= (a * x) % m <= hi`, or `None`; `a` is reduced first.
///
/// cost: O(log m)
/// ties: the least `x`
/// traps: unless `m >= 1` and `0 <= lo <= hi < m`
/// example: `min_mod_in_range(3, 7, 5, 6) == Some(2)`
/// example: `min_mod_in_range(2, 4, 1, 1) == None`
/// example: `min_mod_in_range(5, 9, 0, 3) == Some(0)`
pub fn min_mod_in_range(a: Int, m: Int, lo: Int, hi: Int) -> ?Int:
    assert m >= 1 and 0 <= lo and lo <= hi and hi < m
    min_mod_in_range_reduced((a % m + m) % m, m, lo, hi)

fn min_mod_in_range_reduced(a: Int, m: Int, lo: Int, hi: Int) -> ?Int:
    if lo == 0: return Some(0)
    if a == 0: return None
    let x = (lo + a - 1) / a
    if a * x <= hi: return Some(x)
    let y = min_mod_in_range_reduced(m % a, a, (a - hi % a) % a, (a - lo % a) % a)?
    Some((lo + m * y + a - 1) / a)

/// The continued fraction `[a0, a1, ...]` of `p / q`, `q > 0`, by the Euclidean algorithm with
/// floor division.
///
/// `a0` may be negative or zero; every later term is positive, and the last term is at least 2
/// unless the expansion has one term.
///
/// cost: O(log min(|p|, |q|))
/// order: `a0` first; a new `List`, keyed from 0
/// traps: when `q <= 0`
/// example: `continued_fraction(415, 93) == [4, 2, 6, 7]`
/// example: `continued_fraction(-7, 3) == [-3, 1, 2]`
/// example: `continued_fraction(0, 5) == [0]`
pub fn continued_fraction(p: Int, q: Int) -> List<Int>:
    assert q > 0
    var out: List<Int> = []
    var (x, y) = (p, q)
    while y != 0:
        let t = x / y
        out.push(t)
        (x, y) = (y, x - t * y)
    out

/// The reduced fraction `p / q` with `1 <= q <= max_den` closest to `num / den` (Stern-Brocot).
///
/// cost: O(log max_den)
/// ties: the smaller denominator, then the smaller value
/// traps: when `den == 0` or `max_den < 1`
/// example: `best_rational(31_416, 10_000, 10) == Ratio(22, 7)`
/// example: `best_rational(1, 2, 10) == Ratio(1, 2)`
/// example: `best_rational(1, 2, 1) == Ratio(0, 1)`
pub fn best_rational(num: Int, den: Int, max_den: Int) -> Ratio:
    assert den != 0 and max_den >= 1
    let x = Ratio.new(num, den)
    if x.q <= max_den: return x
    var (p0, q0, p1, q1) = (0, 1, 1, 0)
    var (n, d) = (x.p, x.q)
    while true:
        let a = n / d
        let q2 = q0 + a * q1
        if q2 > max_den: break
        (p0, q0, p1, q1) = (p1, q1, p0 + a * p1, q2)
        (n, d) = (d, n - a * d)
    let k = (max_den - q0) / q1
    let lo = Ratio.new(p0 + k * p1, q0 + k * q1)
    let hi = Ratio.new(p1, q1)
    let dlo = (x - lo).abs
    let dhi = (x - hi).abs
    if dlo < dhi: return lo
    if dhi < dlo: return hi
    if lo.q != hi.q: return if lo.q < hi.q: lo else: hi
    if lo < hi: lo else: hi

/// O(1) storage. Exact reduced rational `p / q` (`q > 0`, `gcd(|p|, q) == 1`).
pub struct Ratio(p: Int, q: Int):
    /// The fraction `p / q` in lowest terms, with a positive denominator.
    ///
    /// cost: O(log min(|p|, |q|))
    /// traps: when `q == 0`
    /// example: `Ratio(2, 4) == Ratio(1, 2)`
    /// example: `Ratio(3, -6).floor == -1`
    /// example: `Ratio(5) == Ratio.from_int(5)`
    pub fn new(p: Int, q: Int = 1) -> Ratio:
        assert q != 0
        let g = gcd(p, q)
        if q < 0: Ratio(-p / g, -q / g) else: Ratio(p / g, q / g)

    /// `n` as the fraction `n / 1`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Ratio.from_int(3) == Ratio(3, 1)`
    pub fn from_int(n: Int) -> Ratio = Ratio.new(n, 1)

    /// The multiplicative inverse `q / p`, sign moved to the numerator.
    ///
    /// cost: O(1)
    /// traps: when `p == 0`
    /// example: `Ratio(-2, 3).inv == Ratio(-3, 2)`
    /// example: `Ratio(1, 4).inv == Ratio.from_int(4)`
    @get pub fn inv(self) -> Ratio = Ratio.new(q, p)

    /// The floor of `p / q` (Euclidean division).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Ratio(7, 2).floor == 3`
    /// example: `Ratio(-7, 2).floor == -4`
    @get pub fn floor(self) -> Int = p / q

    /// The ceiling of `p / q`: `ceil_div(p, q)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Ratio(7, 2).ceil == 4`
    /// example: `Ratio(-7, 2).ceil == -3`
    @get pub fn ceil(self) -> Int = (p + q - 1) / q

    /// The absolute value `|p| / q`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Ratio(-1, 2).abs == Ratio(1, 2)`
    @get pub fn abs(self) -> Ratio = Ratio.new(abs(p), q)

impl Add for Ratio:
    fn add(a, b) = Ratio.new(a.p * b.q + b.p * a.q, a.q * b.q)

impl Sub for Ratio:
    fn sub(a, b) = Ratio.new(a.p * b.q - b.p * a.q, a.q * b.q)

impl Neg for Ratio:
    fn neg(a) = Ratio.new(-a.p, a.q)

impl Mul for Ratio:
    fn mul(a, b) = Ratio.new(a.p * b.p, a.q * b.q)

impl Div for Ratio:
    fn div(a, b) = Ratio.new(a.p * b.q, a.q * b.p)

impl Ord for Ratio:
    fn lt(a, b) = a.p * b.q < b.p * a.q

impl Show for Ratio:
    fn show(a) = "{a.p}/{a.q}"

/// O(n) storage in base-10^9 limbs. Arbitrary-precision signed integer.
pub struct BigInt(neg: Bool, limbs: List<Int>):
    /// `x` as a `BigInt`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `BigInt.from_int(-42).to_str() == "-42"`
    pub fn from_int(x: Int) -> BigInt = todo

    /// Parse a decimal integer: an optional leading `-`, then one or more digits.
    ///
    /// Leading zeros are allowed, and `-0` is zero. There is no `+` and no whitespace.
    ///
    /// cost: O(s.len)
    /// traps: when `s` is not an optional `-` followed by at least one decimal digit
    /// example: `BigInt.from_str("-000123").to_str() == "-123"`
    /// example: `BigInt.from_str("-0").to_str() == "0"`
    /// example: `BigInt.from_str("123456789012345678901234567890").to_str() == "123456789012345678901234567890"`
    pub fn from_str(s: Str) -> BigInt = todo

    /// The decimal text of `self`: no leading zeros, a `-` only when negative, `"0"` for zero.
    ///
    /// cost: O(n)
    /// traps: never
    /// example: `BigInt.from_int(0).to_str() == "0"`
    /// example: `BigInt.from_int(-7).to_str() == "-7"`
    pub fn to_str(self) -> Str = todo

    /// The Euclidean quotient and remainder `(self / d, self % d)`: `0 <= r < |d|`, as for `Int`.
    ///
    /// cost: O(n * m)
    /// traps: when `d` is zero
    /// example: `BigInt.from_int(-7).divmod(BigInt.from_int(3)) == (BigInt.from_int(-3), BigInt.from_int(2))`
    /// example: `BigInt.from_int(7).divmod(BigInt.from_int(-3)) == (BigInt.from_int(-2), BigInt.from_int(1))`
    pub fn divmod(self, d: BigInt) -> (BigInt, BigInt) = todo

    /// The absolute value.
    ///
    /// cost: O(n)
    /// traps: never
    /// example: `BigInt.from_int(-5).abs() == BigInt.from_int(5)`
    pub fn abs(self) -> BigInt = todo

    /// The greatest common divisor of `a` and `b`, never negative; `gcd(0, 0)` is 0.
    ///
    /// cost: O(n^2)
    /// traps: never
    /// example: `BigInt.gcd(BigInt.from_int(12), BigInt.from_int(-18)) == BigInt.from_int(6)`
    /// example: `BigInt.gcd(BigInt.from_int(0), BigInt.from_int(0)) == BigInt.from_int(0)`
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
fn gcd_lcm_and_bezout():
    for _ in 0..200:
        let (a, b) = (rand(-60, 60), rand(-60, 60))
        let g = gcd(a, b)
        assert g >= 0
        if a != 0 or b != 0:
            assert a % g == 0 and b % g == 0
            assert max(d for d in 1..=60 if a % d == 0 and b % d == 0) == g
        let e = ext_gcd(a, b)
        assert e.g == g and a * e.x + b * e.y == g
    assert gcd(0, 0) == 0 and lcm(4, 6) == 12 and lcm(0, 5) == 0 and lcm(-4, 6) == 12

@test
fn choose_is_pascals_triangle():
    assert choose(0, 0) == 1 and choose(5, 2) == 10 and choose(5, 7) == 0 and choose(5, -1) == 0
    assert choose(1000000, 3) == 166666166667000000
    for n in 1..=30:
        for k in 0..=n: assert choose(n, k) == choose(n - 1, k - 1) + choose(n - 1, k)

@test
fn division_helpers_round_the_right_way():
    assert -7 / 2 == -4 and -7 % 2 == 1
    assert 7 / -2 == -3 and 7 % -2 == 1
    assert ceil_div(7, -2) == -3 and ceil_div(-7, 2) == -3
    assert ceil_div(
        -7,
        -2,
    ) == 4 and ceil_div(6, 3) == 2 and ceil_div(6, -3) == -2 and ceil_div(7, 2) == 4
    for _ in 0..200:
        let (a, b) = (rand(-50, 50), rand(1, 9) * (rand(0, 1) * 2 - 1))
        assert a == b * (a / b) + a % b and 0 <= a % b < abs(b)
        let (q, r) = a.divmod(b)
        assert (q, r) == divmod(
            a,
            b,
        ) and q == a / b and r == a % b and 0 <= r < abs(b) and q * b + r == a

@test
fn sieve_and_factorizations_agree():
    let s = Sieve(200)
    assert s.primes[0..10] == [2, 3, 5, 7, 11, 13, 17, 19, 23, 29]
    assert s.primes.len == 46
    for x in 1..=200:
        assert s.factor(x) == factor(x)
        var back = 1
        for (p, e) in s.factor(x):
            assert s.is_prime(p)
            back *= p ** e
        assert back == x
        assert divisors(x) == [d for d in 1..=x if x % d == 0]
    assert not s.is_prime(1) and not s.is_prime(0) and s.is_prime(199)
    assert factor(1000000007 * 2) == [(2, 1), (1000000007, 1)]
    assert bit_len(0) == 0 and bit_len(1) == 1 and bit_len(8) == 4 and bit_len(15) == 4

@test
fn crt_combines_or_refuses():
    for m1 in 1..=12:
        for m2 in 1..=12:
            for _ in 0..4:
                let (r1, r2) = (rand(0, m1 - 1), rand(0, m2 - 1))
                let want = [x for x in 0..lcm(m1, m2) if x % m1 == r1 and x % m2 == r2]
                let got = crt(r1, m1, r2, m2)
                if want.is_empty: assert got == None
                else: assert got == (want[0], lcm(m1, m2))
    assert crt(2, 1000000007, 3, 998244353)!.0 % 998244353 == 3

@test
fn fixed_rounds_pads_and_drops_a_zero_sign():
    assert fixed(1, 3, 5) == "0.33333" and fixed(2, 3, 5) == "0.66667"
    assert fixed(-1, 3, 2) == "-0.33" and fixed(1, -3, 2) == "-0.33"
    assert fixed(-1, 300, 2) == "0.00" and fixed(0, 7, 3) == "0.000"
    assert fixed(7, 2, 0) == "4" and fixed(7, 2, 1) == "3.5" and fixed(-7, 2, 0) == "-4"
    assert fixed(1, 8, 2) == "0.13" and fixed(100, 1, 3) == "100.000"
    assert fixed(5, 1000, 2) == "0.01" and fixed(123456, 1000, 1) == "123.5"

@test
fn bit_reads_one_bit():
    assert 10.bit(1) and 10.bit(3)
    assert not 10.bit(0) and not 10.bit(2) and not 10.bit(40) and not 10.bit(64)
    for _ in 0..100:
        let x = rand(0, 1 << 40)
        assert sum(1 << i for i in 0..41 if x.bit(i)) == x

@test
fn digits_are_lowest_first():
    assert 1203.digits() == [3, 0, 2, 1] and 0.digits() == [0] and 5.digits(2) == [1, 0, 1]
    assert 7 in 1700.digits() and not (7 in 1800.digits())
    for _ in 0..100:
        let (x, base) = (rand(0, 1 << 40), rand(2, 16))
        let ds = x.digits(base)
        assert sum(ds[i] * base ** i for i in 0..ds.len) == x and all(0 <= d < base for d in ds)

@test
fn sign_is_minus_one_zero_or_one():
    assert 5.sign == 1 and (-5).sign == -1 and 0.sign == 0 and (3 - 9).sign == -1

@test
fn is_pow2_is_one_set_bit():
    assert 1.is_pow2 and 2.is_pow2 and 1024.is_pow2 and (1 << 62).is_pow2
    assert not 0.is_pow2 and not 6.is_pow2 and not (-4).is_pow2 and not 1023.is_pow2

@test
fn splitmix64_matches_the_reference_values():
    assert splitmix64(0) == 0xe220a8397b1dcdaf
    assert splitmix64(0x9e3779b97f4a7c15) == 0x6e789e6aa1b965f4
    assert splitmix64(1) != splitmix64(2)

@test
fn miller_rabin_agrees_with_the_sieve_and_refuses_pseudoprimes():
    let s = Sieve(5000)
    for x in -3..5000: assert is_prime(x) == (x >= 2 and s.is_prime(x))
    assert is_prime(998244353) and is_prime(1_000_000_007) and is_prime(1_000_000_000_000_000_003)
    assert is_prime(9_223_372_036_854_775_783)
    assert not is_prime(1_000_000_007 * 998244353) and not is_prime(1_000_000_000_000_000_001)
    for x in [561, 41041, 825265, 3215031751, 2152302898747, 3474749660383, 341550071728321]:
        assert not is_prime(x)

@test
fn factor_agrees_with_sieve_and_handles_big_semiprimes():
    let s = Sieve(2000)
    for x in 1..2000: assert factor(x) == s.factor(x)
    for _ in 0..100:
        let x = rand(1, 1_000_000_000_000)
        let f = factor(x)
        assert all(is_prime(q) for (q, _) in f)
        var back = 1
        for (q, e) in f: back *= q ** e
        assert back == x
    assert factor(1_000_003 * 998244353) == [(1_000_003, 1), (998244353, 1)]
    assert factor(999_999_937 * 999_999_929) == [(999_999_929, 1), (999_999_937, 1)]
    assert factor(2 ** 20 * 3 ** 5 * 10007 ** 2) == [(2, 20), (3, 5), (10007, 2)]
    assert factor(2 ** 62) == [(2, 62)]
    assert factor(1) == []
    for x in [9_223_372_036_854_775_783, 3_037_000_493 * 3_037_000_453, 9_223_372_036_854_775_807]:
        let f = factor(x)
        assert all(is_prime(q) for (q, _) in f)
        var back = 1
        for (q, e) in f: back *= q ** e
        assert back == x

@test
fn pow_mod_phi_and_primitive_roots():
    assert pow_mod(2, 10, 1000) == 24 and pow_mod(5, 0, 1) == 0 and pow_mod(-2, 3, 7) == 6
    assert pow_mod(3, 1_000_000_000_000_000_000, 1_000_000_000_000_000_003) == pow_mod(
        3,
        1_000_000_000_000_000_000 % 1_000_000_000_000_000_002,
        1_000_000_000_000_000_003,
    )
    let s = Sieve(300)
    for x in 1..300:
        assert phi(x) == count(gcd(x, k) == 1 for k in 1..=x)
        assert s.phi(x) == phi(x)
        let f = factor(x)
        let mu = if f.any(fe => fe.1 > 1): 0 elif f.len % 2 == 0: 1 else: -1
        assert s.mobius(x) == mu
    assert primitive_root(998244353) == 3 and primitive_root(1_000_000_007) == 5 and primitive_root(2) == 1
    for p in [3, 5, 7, 11, 13, 41, 71]:
        let g = primitive_root(p)
        assert all(pow_mod(g, (p - 1) / q, p) != 1 for (q, _) in factor(p - 1))

@test
fn floor_sum_agrees_with_the_loop_for_every_sign():
    for _ in 0..400:
        let (n, m, a, b) = (rand(0, 30), rand(1, 20), rand(-40, 40), rand(-40, 40))
        assert floor_sum(n, m, a, b) == sum((a * i + b) / m for i in 0..n)
    assert floor_sum(1_000_000_000, 1_000_000_007, 998_244_353, 123_456_789) == 499122172130479402

@test
fn choose2_inv_mod_crt_all_and_divisors_from_factors():
    assert choose2(-1) == 0 and choose2(0) == 0 and choose2(1) == 0
    assert choose2(2) == 1 and choose2(10) == 45
    assert inv_mod(3, 10) == 7 and inv_mod(2, 10) == None and inv_mod(0, 1) == 0
    assert inv_mod(-3, 10) == 3
    assert crt_all([]) == (0, 1)
    assert crt_all([(2, 3), (3, 5), (2, 7)]) == (23, 105)
    assert crt_all([(1, 4), (3, 6)]) == (9, 12)
    assert crt_all([(1, 4), (2, 6)]) == None
    for x in 1..=100: assert divisors_from_factors(factor(x)) == divisors(x)

@test
fn sieve_tables_and_multiplicative_agree():
    let s = Sieve(300)
    let phis = s.phi_table()
    let mus = s.mobius_table()
    let dcnt = s.div_count_table()
    let dsum = s.div_sum_table()
    let mult_phi = sieve_multiplicative(300, (p, e, pe) => pe - pe / p)
    let mult_d = sieve_multiplicative(300, (p, e, pe) => e + 1)
    for x in 1..=300:
        assert phis[x] == s.phi(x) and mult_phi[x] == phis[x]
        assert mus[x] == s.mobius(x)
        let ds = divisors(x)
        assert dcnt[x] == ds.len and mult_d[x] == dcnt[x]
        assert dsum[x] == sum(ds)

@test
fn mod_log_and_mod_sqrt_match_brute_force():
    for m in [1, 2, 6, 12, 13, 24, 35, 37]:
        for a in 0..min(m, 10):
            for b in 0..min(m, 10):
                let got = mod_log(a, b, m)
                var want: ?Int = None
                var cur = 1 % m
                for x in 0..=m:
                    if cur == b % m:
                        want = x
                        break
                    cur = cur * a % m
                assert got == want
    assert mod_log(2, pow_mod(2, 123456, 1_000_003), 1_000_003) == 123456
    for p in [2, 3, 5, 7, 11, 13, 17, 29, 37, 41, 97]:
        for a in 0..p:
            let r = mod_sqrt(a, p)
            let has = (0..p).any(x => x * x % p == a)
            if has:
                assert r != None and r! * r! % p == a
            else:
                assert r == None
    let big_r = mod_sqrt(2, 998_244_353)
    assert big_r != None and mul_mod(big_r!, big_r!, 998_244_353) == 2
    let big_r2 = mod_sqrt(3, 998_244_353)
    assert big_r2 == None
    let big_r3 = mod_sqrt(9, 998_244_353)
    assert big_r3 != None and mul_mod(big_r3!, big_r3!, 998_244_353) == 9
    let big_r4 = mod_sqrt(mul_mod(1234567, 1234567, 998_244_353), 998_244_353)
    assert big_r4 != None and mul_mod(big_r4!, big_r4!, 998_244_353) == mul_mod(
        1234567,
        1234567,
        998_244_353,
    )

/// `Int.MAX`, for `Int` tables that start "unreached" when `?Int` with `None` is not wanted.
/// `min` treats it as its identity; adding to it overflows (a trap in checked builds).
pub const inf: Int = 9_223_372_036_854_775_807
