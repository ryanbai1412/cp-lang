// Source: Codeforces 2115A (Gellyfish and Flaming Peony),
//   https://codeforces.com/contest/2115/problem/A
// Problem: n <= 5000 values a[i] in 1..=5000. One operation sets a[i] = gcd(a[i], a[j]) for
//   some i != j. Print the fewest operations that make all elements equal. Multitest, sum of
//   n <= 5000.
// Solution: the overall gcd g never changes, so all end at g, and once one element is g each
//   other takes one operation: if g occurs the answer is n minus its count, else
//   (m - 1) + (n - 1) with m the fewest elements whose gcd is g. fewest[d] is a shortest path
//   over values: taking x from d reaches gcd(d, x) with one more element; settle d from
//   largest to smallest, O(max * distinct values).
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
