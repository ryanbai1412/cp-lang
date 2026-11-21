//! Number theory on plain `Int`s.

/// The greatest common divisor, never negative; `gcd(0, 0) == 0`. O(log min(a, b)).
pub fn gcd(a: Int, b: Int) -> Int:
    var (x, y) = (abs(a), abs(b))
    while y != 0: (x, y) = (y, x % y)
    x

/// The least common multiple, never negative; stops on overflow.
pub fn lcm(a: Int, b: Int) -> Int = if a == 0 or b == 0: 0 else: abs(a / gcd(a, b) * b)

/// `num / den` with exactly `k` digits after the point, as text, from
/// integers alone: no float rounding. Halves round away from zero, and a
/// result that rounds to zero has no sign: `fixed(-1, 300, 2) == "0.00"`.
/// `fixed(t, 2, 1)` prints a half-integer. `|num| * 10^k * 2` must fit an `Int`.
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
/// O(min(k, n - k)); stops on overflow. For counts modulo a prime use `Comb` in `std.mod`.
pub fn choose(n: Int, k: Int) -> Int:
    if k < 0 or k > n: return 0
    var r = 1
    for i in 0..min(k, n - k): r = r * (n - i) / (i + 1)
    r

/// `(g, x, y)` with `a * x + b * y == g == gcd(a, b)`. O(log min(a, b)).
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

/// `ceil(a / b)` for any non-zero `b`. Built-in `/` is Euclidean division
/// (remainder is always non-negative), which already equals `floor(a / b)` for `b > 0`.
pub fn ceil_div(a: Int, b: Int) -> Int = if b > 0: -((-a) / b) else: -(a / (-b))

/// The smallest prime factor of every number up to `n` (0 for 0 and 1), by a linear sieve.
/// O(n) time and memory.
pub struct Sieve:
    spf: List<Int>
    primes: List<Int>

    pub fn new(n: Int) -> Sieve:
        var spf = List.filled(n + 1, 0)
        var primes = List<Int>()
        for i in 2..=n:
            if spf[i] == 0:
                spf[i] = i
                primes.push(i)
            for p in primes:
                if p > spf[i] or i * p > n: break
                spf[i * p] = p
        Sieve(spf, primes)

    pub fn is_prime(self, x: Int) -> Bool = x >= 2 and spf[x] == x

    /// The prime factorization of `x >= 1` as `(prime, exponent)` pairs, smallest prime first. O(log x).
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

    /// Euler's totient of `1 <= x <= n`. O(log x).
    pub fn phi(self, x: Int) -> Int:
        var acc = x
        for (p, _) in self.factor(x): acc = acc / p * (p - 1)
        acc

    /// The Mobius function of `1 <= x <= n`: 0 if a square divides `x`, else -1 to the
    /// number of prime factors. O(log x).
    pub fn mobius(self, x: Int) -> Int:
        let f = self.factor(x)
        if any(e > 1 for (_, e) in f): 0 elif f.len % 2 == 0: 1 else: -1

/// The number of bits needed to write `x >= 0` (`bit_len(0) == 0`, `bit_len(5) == 3`).
pub fn bit_len(x: Int) -> Int = bit_width(x)

/// The digits of `x >= 0` in `base`, lowest first: `1203.digits()` is `[3, 0, 2, 1]`, and `0` has the one digit `0`.
pub fn digits(self: Int, base: Int = 10) -> List<Int>:
    assert self >= 0 and base >= 2
    var (out, x) = ([self % base], self / base)
    while x > 0:
        out.push(x % base)
        x /= base
    out

/// -1, 0 or 1 as `x` is below, at or above zero: `(b - a).sign`.
pub @get fn sign(self: Int) -> Int = if self > 0: 1 elif self < 0: -1 else: 0

/// Whether bit `i` of `x` is set: `x.bit(i)`, bit 0 the lowest. No bit from 63 up is set in an `Int` that is not negative.
pub fn bit(self: Int, i: Int) -> Bool = i < 64 and (self >> i) & 1 == 1

/// Is `n` a power of two? `8.is_pow2` is true; `0`, `6` and negatives are not.
pub @get fn is_pow2(self: Int) -> Bool = self > 0 and self & (self - 1) == 0

/// SplitMix64: mixes the 64 bits of `x` so that inputs close together land far apart. The
/// usual guard for `HashMap` keys and the step of a seeded generator; `U64` arithmetic wraps.
pub fn splitmix64(x: U64) -> U64:
    var z: U64 = x + 0x9e3779b97f4a7c15
    z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9
    z = (z ^ (z >> 27)) * 0x94d049bb133111eb
    z ^ (z >> 31)

/// The prime factorization of `x >= 1` by trial division. O(sqrt x).
pub fn factor(x: Int) -> List<(Int, Int)>:
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
    out

/// The divisors of `x >= 1`, sorted. O(sqrt x).
pub fn divisors(x: Int) -> List<Int>:
    var (small, large) = (List<Int>(), List<Int>())
    var d = 1
    while d * d <= x:
        if x % d == 0:
            small.push(d)
            if d * d != x: large.push(x / d)
        d += 1
    small + large.rev

/// The solution of `x = r1 (mod m1)`, `x = r2 (mod m2)` as `(r, m)` with
/// `m = lcm(m1, m2)` and `0 <= r < m`, or `none` if there is none.
pub fn crt(r1: Int, m1: Int, r2: Int, m2: Int) -> ?(Int, Int):
    let e = ext_gcd(m1, m2)
    if (r2 - r1) % e.g != 0: return none
    let m = m1 / e.g * m2
    let step = m2 / e.g
    // x = r1 + m1 * k with k = (r2 - r1) / g * inv(m1 / g) mod (m2 / g)
    let k = mul_mod((r2 - r1) / e.g % step, e.x % step, step)
    ((r1 + m1 * k) % m, m)

/// `a * b % m` without overflow, rounded down to `0..m` for `m >= 1`. O(1), through `I128`
/// when the product could leave `Int`.
pub fn mul_mod(a: Int, b: Int, m: Int) -> Int:
    if -3_037_000_499 <= a <= 3_037_000_499 and -3_037_000_499 <= b <= 3_037_000_499: a * b % m
    else: Int(I128(a) * I128(b) % I128(m))

/// `a ** e % m` for `e >= 0` and `1 <= m`. O(log e) multiplications, each O(1).
pub fn pow_mod(a: Int, e: Int, m: Int) -> Int:
    var (acc, x, k) = (1 % m, a % m, e)
    while k > 0:
        if k % 2 == 1: acc = mul_mod(acc, x, m)
        x = mul_mod(x, x, m)
        k /= 2
    acc

/// Is `x` prime? Deterministic Miller-Rabin over the first twelve primes, exact for every
/// `Int`. O(log^2 x); `Sieve(n).is_prime(x)` is O(1) for many small `x`.
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

/// The prime factorization of `x >= 1` as `(prime, exponent)` pairs, smallest prime first,
/// for any `Int`: trial division below 100, then Pollard's rho. Expected
/// O(x^(1/4) log x); `factor(x)` is simpler for `x` up to about 1e12.
pub fn factor_big(x: Int) -> List<(Int, Int)>:
    assert x >= 1
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

/// Euler's totient of `x >= 1`: how many of `1..=x` are coprime to `x`. O(sqrt x).
pub fn phi(x: Int) -> Int:
    var acc = x
    for (p, _) in factor(x): acc = acc / p * (p - 1)
    acc

/// The smallest primitive root modulo the prime `p`: a `g` whose powers reach every
/// non-zero residue. Expected O(p^(1/4) log p) to factor `p - 1`, then O(log^2 p) per try.
pub fn primitive_root(p: Int) -> Int:
    assert is_prime(p)
    if p == 2: return 1
    let parts = [q for (q, _) in factor_big(p - 1)]
    var g = 2
    while any(pow_mod(g, (p - 1) / q, p) == 1 for q in parts): g += 1
    g

/// The sum of `(a * i + b) / m` over `i` in `0..n`, with floor division, for `n >= 0` and
/// `m >= 1`. O(log m), by the Euclid-like reduction.
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
                if want.is_empty: assert got == none
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
fn factor_big_agrees_with_trial_division_and_handles_big_semiprimes():
    for x in 1..2000: assert factor_big(x) == factor(x)
    for _ in 0..200:
        let x = rand(1, 1_000_000_000_000)
        assert factor_big(x) == factor(x)
    assert factor_big(1_000_003 * 998244353) == [(1_000_003, 1), (998244353, 1)]
    assert factor_big(999_999_937 * 999_999_929) == [(999_999_929, 1), (999_999_937, 1)]
    assert factor_big(2 ** 20 * 3 ** 5 * 10007 ** 2) == [(2, 20), (3, 5), (10007, 2)]
    assert factor_big(2 ** 62) == [(2, 62)]
    assert factor_big(1) == []
    for x in [9_223_372_036_854_775_783, 3_037_000_493 * 3_037_000_453, 9_223_372_036_854_775_807]:
        let f = factor_big(x)
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
        let mu = if any(e > 1 for (_, e) in f): 0 elif f.len % 2 == 0: 1 else: -1
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
