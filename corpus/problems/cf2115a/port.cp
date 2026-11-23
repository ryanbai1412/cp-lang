// Codeforces 2115 A: Gellyfish and Flaming Peony.
//
// An operation a_i = gcd(a_i, a_j) never changes the overall gcd g, and the
// elements must end equal, so they all end at g. Once one element equals g,
// every other element takes one operation. So if g already occurs, the
// answer is n minus its count; otherwise it is (m - 1) + (n - 1), where m is
// the fewest elements whose gcd is g: folding them into one costs m - 1.
//
// fewest[d], the fewest elements with gcd d, is a shortest path over values:
// taking x from d reaches gcd(d, x) <= d with one more element. Settle d from
// largest to smallest. O(max * distinct values).
@!multitest
let n: 1..=5000 = read()
type Val = 1..=5000
let a: [0..n]Val = read()
let g = a.fold(gcd)
let at_g = a.count(g)
if at_g > 0: return print(n - at_g)

let values = a.distinct()
var fewest: [Val]?Int = None
for x in values: fewest[x] = 1
for d in (1..=a.max()!).rev:
    let k = fewest[d] ?? continue
    for x in values:
        fewest[gcd(d, x)] min= k + 1
print(fewest[g]! - 1 + n - 1)
