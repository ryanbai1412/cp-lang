//! Primality and factorization for numbers too large to sieve: a deterministic
//! Miller-Rabin test and Pollard's rho, for any `n` below `4 * 10 ** 18`.
//!
//! `std.math.Sieve` and `std.math.factor` cover small numbers; these take over when
//! the square root no longer fits in the time limit.
use std.math.{gcd, mul_mod}

/// The largest `n` the functions here accept: sums of two residues must fit an `Int`.
pub const LIMIT = 4_000_000_000_000_000_000

/// `a * b % m` for `0 <= a, b < m`: one machine multiplication when it cannot overflow.
fn mul(a: Int, b: Int, m: Int) -> Int = if m <= 3_000_000_000: a * b % m else: mul_mod(a, b, m)

/// `a ** e % m` for `e >= 0`. O(log e) multiplications.
pub fn pow_mod(a: Int, e: Int, m: Int) -> Int:
    var (acc, x, k) = (1 % m, a % m, e)
    while k > 0:
        if k % 2 == 1: acc = mul(acc, x, m)
        x = mul(x, x, m)
        k /= 2
    acc

/// Is `n` prime? Exact for every `n <= LIMIT` (Miller-Rabin over the first twelve
/// primes, which is known to have no false positives below 3.3 * 10 ** 24).
pub fn is_prime(n: Int) -> Bool:
    assert n <= LIMIT
    if n < 2: return false
    let bases = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37]
    for p in bases:
        if n % p == 0: return n == p
    // n - 1 = odd * 2 ** twos
    var (odd, twos) = (n - 1, 0)
    while odd % 2 == 0:
        odd /= 2
        twos += 1
    for a in bases:
        var x = pow_mod(a, odd, n)
        if x == 1 or x == n - 1: continue
        var witness = true
        for _ in 1..twos:
            x = mul(x, x, n)
            if x == n - 1:
                witness = false
                break
        if witness: return false
    true

/// A divisor of the composite `n` other than 1 and `n` (Pollard's rho with Floyd's
/// cycle detection). Expected O(n ** 0.25) multiplications.
fn split(n: Int) -> Int:
    if n % 2 == 0: return 2
    var c = 1
    while true:
        var (x, y, d) = (2, 2, 1)
        while d == 1:
            x = (mul(x, x, n) + c) % n
            y = (mul(y, y, n) + c) % n
            y = (mul(y, y, n) + c) % n
            d = gcd(abs(x - y), n)
        if d != n: return d
        c += 1                                       // the walk closed on itself: try another
    n

/// The prime factorization of `n >= 1` as `(prime, exponent)` pairs, primes ascending:
/// the shape of `std.math.factor`, without its O(sqrt n) cost.
pub fn factorize(n: Int) -> List<(Int, Int)>:
    assert 1 <= n <= LIMIT
    var primes: List<Int> = []
    var todo = [n]
    while let m = todo.pop():
        if m == 1: continue
        if is_prime(m):
            primes.push(m)
        else:
            let d = split(m)
            todo.push(d)
            todo.push(m / d)
    sort(mut primes)
    var found: List<(Int, Int)> = []
    for p in primes:
        if not found.is_empty and found[found.len - 1].0 == p: found[found.len - 1] = (p, found[found.len - 1].1 + 1)
        else: found.push((p, 1))
    found

/// Every divisor of `n >= 1`, ascending. O(number of divisors) after `factorize`.
pub fn divisors_of(n: Int) -> List<Int>:
    var ds = [1]
    for (p, e) in factorize(n):
        let base = [d for d in ds]
        var scale = 1
        for _ in 0..e:
            scale *= p
            for d in base: ds.push(d * scale)
    sort(mut ds)
    ds

/// The smallest primitive root modulo the prime `p`: a `g` whose powers reach every
/// non-zero residue.
pub fn primitive_root(p: Int) -> Int:
    assert is_prime(p)
    if p == 2: return 1
    let parts = [q for (q, _) in factorize(p - 1)]
    var g = 2
    while any(pow_mod(g, (p - 1) / q, p) == 1 for q in parts): g += 1
    g

@test
fn small_numbers_agree_with_trial_division():
    for n in 0..600:
        assert is_prime(n) == (n >= 2 and all(n % d != 0 for d in 2..n if d * d <= n))
    for n in 1..400:
        let f = factorize(n)
        assert all(is_prime(p) and e >= 1 for (p, e) in f)
        assert [p for (p, _) in f] == sorted([p for (p, _) in f])
        var back = 1
        for (p, e) in f: back *= p ** e
        assert back == n
        assert divisors_of(n) == [d for d in 1..=n if n % d == 0]

@test
fn large_primes_composites_and_strong_pseudoprimes():
    assert is_prime(998244353) and is_prime(1_000_000_007) and is_prime(1_000_000_000_000_000_003)
    assert not is_prime(1_000_000_007 * 998244353) and not is_prime(1_000_000_000_000_000_001)
    // Carmichael numbers and strong pseudoprimes to several small bases
    for n in [561, 41041, 825265, 3215031751, 2152302898747, 3474749660383, 341550071728321]:
        assert not is_prime(n)
    assert factorize(1_000_003 * 998244353) == [(1_000_003, 1), (998244353, 1)]
    assert factorize(2 ** 20 * 3 ** 5 * 10007 ** 2) == [(2, 20), (3, 5), (10007, 2)]
    assert factorize(999_983 * 999_983 * 1_000_003) == [(999_983, 2), (1_000_003, 1)]
    assert divisors_of(2 ** 61).len == 62

@test
fn primitive_roots_generate_everything():
    assert primitive_root(998244353) == 3 and primitive_root(1_000_000_007) == 5 and primitive_root(2) == 1
    for p in [3, 5, 7, 11, 13, 41, 71]:
        let g = primitive_root(p)
        var (x, seen) = (1, List.filled(p, false))
        for _ in 1..p:
            x = x * g % p
            seen[x] = true
        assert count(s for s in seen) == p - 1
