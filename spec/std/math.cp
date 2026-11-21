//! Number theory on plain `Int`s.

/// The greatest common divisor, never negative; `gcd(0, 0) == 0`. O(log min(a, b)).
pub fn gcd(a: Int, b: Int) -> Int:
    var (x, y) = (abs(a), abs(b))
    while y != 0: (x, y) = (y, x % y)
    x

/// The least common multiple, never negative; stops on overflow.
pub fn lcm(a: Int, b: Int) -> Int = if a == 0 or b == 0: 0 else: abs(a / gcd(a, b) * b)

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

/// `n * (n - 1) / 2` for `n >= 2`, else `0`. O(1).
pub fn choose2(n: Int) -> Int = if n < 2: 0 else: n * (n - 1) / 2

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

/// Euclidean quotient and remainder `(self / d, self % d)` (`0 <= r < |d|`,
/// `self == q * d + r`; traps on `d == 0`). Works as both `n.divmod(d)` and `divmod(n, d)`. O(1).
pub fn divmod(self: Int, d: Int) -> (Int, Int) = (self / d, self % d)

/// The smallest prime factor of every number up to `n` (0 for 0 and 1), by a linear sieve.
/// O(n) time and memory.
pub struct Sieve:
    spf: List<0..=2_000_000_000>
    primes: List<0..=2_000_000_000>

    /// Linear sieve up to `n`, populating `spf` and `primes`. O(n) time and space.
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

    /// Whether `x` is prime (for `0 <= x <= n`). O(1).
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

    /// Table of Euler's totient `phi[0..=n]`. O(n).
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

    /// Table of the Mobius function `mu[0..=n]` (`mu[0] == 0`, `mu[1] == 1`). O(n).
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

    /// Table of divisor counts `d[0..=n]`. O(n).
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

    /// Table of divisor sums `sigma[0..=n]`. O(n).
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

    /// O(d(x) log d(x)). Sorted positive divisors of `1 <= x <= n`.
    pub fn divisors(self, x: Int) -> List<Int> = todo

    /// O(d(x) + log x). Generator yielding each positive divisor of `1 <= x <= n`.
    pub fn each_divisor(self, x: Int) -> yields Int = todo

/// Tabulate any multiplicative function `f` on `1..=n` in O(n) via a linear sieve.
/// `f(1) = T.one`, `f(a * b) = f(a) * f(b)` for coprime `a, b`, and
/// `f(p^e) = prime_pow(p, e, p^e)`. Index 0 holds `T.one`.
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

/// The prime factorization of `x >= 1` as `(prime, exponent)` pairs, smallest prime first,
/// for any 64-bit `Int`: trial division for `x <= 1_000_000` (O(sqrt x)), Pollard's rho +
/// deterministic Miller-Rabin above (O(x^(1/4) log x) expected).
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

/// All positive divisors of the integer with prime factorization `factors`, sorted. O(d log d).
pub fn divisors_from_factors(factors: List<(Int, Int)>) -> List<Int>:
    var out = [1]
    for (p, e) in factors:
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

/// The modular inverse of `a` modulo `m >= 1`, in `0..m`, or `none` if `gcd(a, m) != 1`. O(log m).
pub fn inv_mod(a: Int, m: Int) -> ?Int:
    assert m >= 1
    let (g, x) = ext_gcd_x(a % m, m)
    if g != 1: return none
    x % m

/// The solution of `x = r1 (mod m1)`, `x = r2 (mod m2)` as `(r, m)` with
/// `m = lcm(m1, m2)` and `0 <= r < m`, or `none` if there is none.
pub fn crt(r1: Int, m1: Int, r2: Int, m2: Int) -> ?(Int, Int):
    let (g, x) = ext_gcd_x(m1, m2)
    if (r2 - r1) % g != 0: return none
    let m = m1 / g * m2
    let step = m2 / g
    // x = r1 + m1 * k with k = (r2 - r1) / g * inv(m1 / g) mod (m2 / g)
    let k = mul_mod((r2 - r1) / g % step, x % step, step)
    ((r1 + m1 * k) % m, m)

/// The solution of a system of congruences `x = r (mod m)` as `(r, m)` with `0 <= r < m`,
/// or `none` if inconsistent. Empty input yields `(0, 1)`. O(eqs.len * log lcm).
pub fn crt_all(eqs: List<(Int, Int)>) -> ?(Int, Int):
    var (r, m) = (0, 1)
    for (ri, mi) in eqs:
        let next = crt(r, m, ri, mi)
        if next == none: return none
        (r, m) = next!
    (r, m)

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

/// Euler's totient of `x >= 1`: how many of `1..=x` are coprime to `x`.
/// O(sqrt x) for `x <= 1_000_000`, O(x^(1/4) log x) expected above.
pub fn phi(x: Int) -> Int:
    var acc = x
    for (p, _) in factor(x): acc = acc / p * (p - 1)
    acc

/// The smallest primitive root modulo the prime `p`: a `g` whose powers reach every
/// non-zero residue. O(p^(1/4) log p) expected to factor `p - 1`, then O(log^2 p) per try.
pub fn primitive_root(p: Int) -> Int:
    assert is_prime(p)
    if p == 2: return 1
    let parts = [q for (q, _) in factor(p - 1)]
    var g = 2
    while any(pow_mod(g, (p - 1) / q, p) == 1 for q in parts): g += 1
    g

/// The smallest non-negative `x` with `a^x = b (mod m)`, or `none` if none exists.
/// Handles arbitrary `m >= 1` including non-coprime `a` and `m`. O(sqrt m).
pub fn mod_log(a: Int, b: Int, m: Int) -> ?Int:
    assert m >= 1
    let (a0, b0) = (a % m, b % m)
    if m == 1 or b0 == 1: return 0
    var (k, add, cur_b, cur_m) = (1, 0, b0, m)
    while true:
        let g = gcd(a0, cur_m)
        if g == 1: break
        if k == cur_b: return add
        if cur_b % g != 0: return none
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
    none

/// A square root `x` in `0..p` with `x * x = a (mod p)` for prime `p`, or `none`
/// if `a` is a quadratic non-residue. Tonelli-Shanks, O(log^2 p).
pub fn mod_sqrt(a: Int, p: Int) -> ?Int:
    assert is_prime(p)
    let x = a % p
    if x == 0 or p == 2: return x
    if pow_mod(x, (p - 1) / 2, p) != 1: return none
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
pub fn best_rational(num: Int, den: Int, max_den: Int) -> Frac = todo

/// O(1) storage. Exact reduced rational `p / q` (`q > 0`, `gcd(|p|, q) == 1`).
pub struct Frac(p: Int, q: Int):
    /// O(log min(|p|, |q|)). Reduced fraction `p / q`; traps on `q == 0`.
    pub fn new(p: Int, q: Int = 1) -> Frac = todo

    /// O(1). Lift an `Int` to `Frac(n, 1)`.
    pub fn from_int(n: Int) -> Frac = todo

    /// O(1). Multiplicative inverse `q / p`; traps on `p == 0`.
    pub @get fn inv(self) -> Frac = todo

    /// O(1). Floor `p / q` (Euclidean).
    pub @get fn floor(self) -> Int = todo

    /// O(1). Ceiling `ceil_div(p, q)`.
    pub @get fn ceil(self) -> Int = todo

    /// O(1). Absolute value `|p| / q`.
    pub @get fn abs(self) -> Frac = todo

impl Add for Frac:
    fn add(a, b) = todo

impl Sub for Frac:
    fn sub(a, b) = todo

impl Neg for Frac:
    fn neg(a) = todo

impl Mul for Frac:
    fn mul(a, b) = todo

impl Div for Frac:
    fn div(a, b) = todo

impl Ord for Frac:
    fn lt(a, b) = todo

impl Show for Frac:
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

@test
fn choose2_inv_mod_crt_all_and_divisors_from_factors():
    assert choose2(-1) == 0 and choose2(0) == 0 and choose2(1) == 0
    assert choose2(2) == 1 and choose2(10) == 45
    assert inv_mod(3, 10) == 7 and inv_mod(2, 10) == none and inv_mod(0, 1) == 0
    assert inv_mod(-3, 10) == 3
    assert crt_all([]) == (0, 1)
    assert crt_all([(2, 3), (3, 5), (2, 7)]) == (23, 105)
    assert crt_all([(1, 4), (3, 6)]) == (9, 12)
    assert crt_all([(1, 4), (2, 6)]) == none
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
                var want: ?Int = none
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
            let has = any(x * x % p == a for x in 0..p)
            if has:
                assert r != none and r! * r! % p == a
            else:
                assert r == none
    let big_r = mod_sqrt(2, 998_244_353)
    assert big_r != none and mul_mod(big_r!, big_r!, 998_244_353) == 2
    let big_r2 = mod_sqrt(3, 998_244_353)
    assert big_r2 == none
    let big_r3 = mod_sqrt(9, 998_244_353)
    assert big_r3 != none and mul_mod(big_r3!, big_r3!, 998_244_353) == 9
    let big_r4 = mod_sqrt(mul_mod(1234567, 1234567, 998_244_353), 998_244_353)
    assert big_r4 != none and mul_mod(big_r4!, big_r4!, 998_244_353) == mul_mod(
        1234567,
        1234567,
        998_244_353,
    )
