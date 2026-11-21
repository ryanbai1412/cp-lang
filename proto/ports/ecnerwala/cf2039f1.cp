// Codeforces 2039 F1: count the good arrays over 1..=m (distinct f(k) = gcd of the window maxima), mod 998244353.
// An array is good iff its values are distinct and, sorted, have strictly growing suffix gcds; each such set
// orders in 2^(n−1) good ways. Add values from m down to 1: an open set (all values above y) with gcd g takes y
// iff gcd(y, g) < g, doubling its weight. By Möbius, the weight landing on h | y is Σ_{d | y/h} μ(d)·above[h·d]
// (above[x] = the open sets whose gcd x divides), less the sets whose gcd is h already.
@!multitest
type Z = Mod<998_244_353>
let m: 1..=100_000 = read()
let mu = Sieve.new(m).mobius_table()
var divs: [m + 1]List<Int> = []
for d in 1..=m:
    for x in (d..=m).step(d): divs[x].push(d)

var at: [m + 1]Z = 0
var above: [m + 1]Z = 0
for y in (1..=m).rev:
    let grown = [(h, 2 * (sum(mu[d] * above[h * d] for d in divs[y / h]) - at[h])) for h in divs[y]]
    for (h, w) in grown + [(y, 1)]:
        at[h] += w
        for d in divs[h]: above[d] += w
print(above[1])
