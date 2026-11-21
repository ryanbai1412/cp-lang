// Codeforces 2039 F1: count the good arrays over 1..=m (distinct f(k) = gcd of the window maxima), mod 998244353.
// An array is good iff its values are distinct and, sorted, have strictly growing suffix gcds; each such set
// orders in 2^(n−1) good ways. Add values from m down to 1: an open set (all values above y) with gcd g takes y
// iff gcd(y, g) < g, doubling its weight. By Möbius, the weight landing on h | y is Σ_{d | y/h} μ(d)·above[h·d]
// (above[x] = the open sets whose gcd x divides), less the sets whose gcd is h already.
use std.math.{Sieve}
use std.mod.{Mod}

@!multitest
type Z = Mod<998_244_353>
let m: 1..=1e5 = read()
domain G = m.divisors
let numbers = Sieve(m)
var at, above: [G]Z = 0
for y in G.rev:
    let grown = [h => 2 * (sum(numbers.mobius(d) * above[h * d] for d in (y / h).divisors) - at[h]) for h in y.divisors]
    for (h, w) in grown + [y: 1]:
        at[h] += w
        above[h.divisors] += w
print(above[G.first])
