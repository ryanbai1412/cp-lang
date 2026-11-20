// cp: 0.1 8ef64b9+dirty
// NOIP 2009 Hankson: count x with gcd(x, a0) = a1 and lcm(x, b0) = b1.
// Independently per prime p: count exponents e of p in x.
const LIM = 44722                           // > sqrt(2e9)
var composite: [0..=LIM]Bool = false
var primes: List<Int> = []
for p in 2..=LIM:
    if not composite[p]:
        primes.push(p)
        var q = p * p
        while q <= LIM:
            composite[q] = true
            q += p

fn prime_factors(x: 1..=2e9) -> yields Int:
    var r = x
    for p in primes:
        if p * p > r:
            break
        if r % p == 0:
            yield p
            while r % p == 0:
                r /= p
    if r > 1:
        yield r

fn vp(x: 1..=2e9, p: 2..=2e9) -> Int:
    var r = x
    var e = 0
    while r % p == 0:
        r /= p
        e += 1
    e

// x | b1, so only primes of b1 can appear in x; primes of a0 can still rule x out
fn support(a: 1..=2e9, b: 1..=2e9) -> yields Int:
    yield from prime_factors(a)
    for p in prime_factors(b):
        if a % p != 0:
            yield p

fn gcd(a: Int, b: Int) -> Int:
    if b == 0:
        return a
    gcd(b, a % b)

struct Query:
    a0, a1, b0, b1: 1..=2e9

// uniform values almost never admit any x; plant one instead
fn plant() -> Query:
    let x = rand(1, 1000)
    let a0 = rand(1, 1000)
    let b0 = rand(1, 1000)
    (a0: a0, a1: gcd(x, a0), b0: b0, b1: x / gcd(x, b0) * b0)

@multitest
fn solve():
    let q: Query = read(gen: plant) where q.a0 % q.a1 == 0 and q.b1 % q.b0 == 0
    var total = 1
    for p in support(q.a0, q.b1):
        let (a, g, b, l) = (vp(q.a0, p), vp(q.a1, p), vp(q.b0, p), vp(q.b1, p))
        total *= count(min(e, a) == g and max(e, b) == l for e in 0..=max(l, a))
    print(total)
