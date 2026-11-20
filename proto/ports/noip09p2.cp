// NOIP 2009 Hankson: count x with gcd(x, a0) = a1 and lcm(x, b0) = b1.
// x divides b1, so try each prime of b1 on its own: count the exponents e
// with min(e, vp(a0)) = vp(a1) and max(e, vp(b0)) = vp(b1).
const LIM = 44722                           // > sqrt(2e9)
var composite: [0..=LIM]Bool = false
var primes: List<Int> = []
for p in 2..=LIM if not composite[p]:
    primes.push(p)
    for q in (p * p..=LIM).step(p): composite[q] = true

// how many times p divides x
fn vp(x: Int, p: Int) -> Int:
    var (x, e) = (x, 0)
    while x % p == 0:
        x /= p
        e += 1
    e

@!multitest
let (a0, a1, b0, b1): (1..=2e9, 1..=2e9, 1..=2e9, 1..=2e9) = read()

fn choices(p: Int) -> Int:
    let (a, g, b, l) = (vp(a0, p), vp(a1, p), vp(b0, p), vp(b1, p))
    count(min(e, a) == g and max(e, b) == l for e in 0..=l)

// a prime of a0 that is not in b1 can't be in x, so a1 must not have it either
if gcd(a0, b1) % a1 != 0:
    print(0)
    return
var total = 1
var rest = b1
for p in primes if rest % p == 0:
    total *= choices(p)
    while rest % p == 0: rest /= p
if rest > 1: total *= choices(rest)
print(total)
