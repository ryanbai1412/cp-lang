// Codeforces 2201 E: ABBA Counting.
// S = A+B+B+A exactly when its right half Y is a rotation of its left half X.
// For each period len | m, fold both halves mod len and count pairs (X, shift k)
// with three cyclic correlations: no a/b clash, then 2^(?-? slots) completions.
// An S recurs once per len its X's period divides; Möbius inversion over the
// divisors weights len by prod (1 - p) over the primes p | m / len.

use std.math.{divisors, prime_factors}
use std.ntt.{correlate}

type Z = Mod<998_244_353>

@!multitest
let n: 2..=400_000 = read()
let t: Str = read()
let m = n / 2

// The fixed letters of one class mod len must agree; none if two clash.
fn fold(half: Str, len: Int) -> ?[0..len]?Char:
    var slot: [0..len]?Char = none
    for (i, c) in half.items:
        if c == '?': continue
        if let prev = slot[i % len], prev != c: return none
        slot[i % len] = c
    slot

fn shifts(len: Int) -> Z:
    let some(x) = fold(t[0..m], len) else: return 0
    let some(y) = fold(t[m..n], len) else: return 0
    let pairs = (u: ?Char, v: ?Char) =>
        correlate(x.map(c => Int(c == u)), y.map(c => Int(c == v)), cyclic: true)
    let (ab, ba, free) = (pairs('a', 'b'), pairs('b', 'a'), pairs(none, none))
    [Z(2) ** free[k] for k in 0..len if ab[k] + ba[k] == 0].sum()

print(divisors(m).sum<Z>(len => shifts(len) * prime_factors(m / len).product<Z>(p => 1 - p)))
