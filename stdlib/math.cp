//! Number theory on plain `Int`s.

/// The greatest common divisor, never negative; `gcd(0, 0) == 0`. O(log min(a, b)).
pub fn gcd(a: Int, b: Int) -> Int:
    var (x, y) = (abs(a), abs(b))
    while y != 0: (x, y) = (y, x % y)
    x

/// The least common multiple, never negative; stops on overflow.
pub fn lcm(a: Int, b: Int) -> Int = if a == 0 or b == 0: 0 else: abs(a / gcd(a, b) * b)

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

/// The number of bits needed to write `x >= 0` (`bit_len(0) == 0`, `bit_len(5) == 3`).
pub fn bit_len(x: Int) -> Int = bit_width(x)

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

/// `a * b % m` without overflow, for `0 <= a, b < m`. O(log b).
pub fn mul_mod(a: Int, b: Int, m: Int) -> Int:
    var (acc, x, k) = (0, a % m, b)
    while k > 0:
        if k % 2 == 1: acc = (acc + x) % m
        x = (x + x) % m
        k /= 2
    acc

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
    assert ceil_div(-7, -2) == 4 and ceil_div(6, 3) == 2 and ceil_div(6, -3) == -2 and ceil_div(7, 2) == 4
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
