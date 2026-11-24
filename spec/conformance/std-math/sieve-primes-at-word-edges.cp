// expect: ok, output "ok"
// A Sieve's primes and is_prime match trial division for every bound up to 400, so across the
// bitset's 64-bit word edges (127, 128, 129, 255, 256, 257, 383, 384, 385).
fn trial(x: Int) -> Bool:
    if x < 2: return false
    var d = 2
    while d * d <= x:
        if x % d == 0: return false
        d += 1
    true
var bad: List<(Int, Int)> = []
for n in 0..=400:
    let s = Sieve(n)
    if s.primes.to_list() != [x for x in 0..=n if trial(x)]: bad.push((n, -1))
    for x in 0..=n:
        if s.is_prime(x) != trial(x): bad.push((n, x))
print(if bad.len == 0: "ok" else: "bad {bad[0]}")
