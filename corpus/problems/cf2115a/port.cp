// Source: Codeforces 2115A (Gellyfish and Flaming Peony),
//   https://codeforces.com/contest/2115/problem/A
// Problem: n <= 5000 values a[i] in 1..=5000. One operation sets a[i] = gcd(a[i], a[j]) for
//   some i != j. Print the fewest operations that make all elements equal. Multitest, sum of
//   n <= 5000.
// Solution: the overall gcd g never changes, so all end at g, and once one element is g each
//   other takes one operation: if g occurs the answer is n minus its count, else
//   (m - 1) + (n - 1) with m the fewest elements whose gcd is g. Divided by g, m is the least k
//   with a k-tuple of gcd 1, and by Mobius sum_d mu(d) c_d^k counts those, c_d the elements
//   divisible by d. Each value weighs a random hash mod p, so the count is nonzero mod p
//   whenever it is truly nonzero (with high probability). O(V log V) per k.
use std.math.{Sieve}
use std.rand.{Rng}

const MOD = 998_244_353
type Z = Mod<MOD>
const V = 5000
let mu = Sieve(V).mobius_table()
var rng = Rng(48)

@!multitest
let n: 1..=5000 = read()
let a: [0..n]1..=V = read()
let g = a.fold(gcd)
let at_g = a.count(g)
if at_g > 0: return print(n - at_g)

var mult: [1..=V]Z = 0
for x in a: mult[x / g] += Z(Int(rng.next() % U64(MOD)))
for d in 1..=V:
    for j in (2 * d..=V).step(d): mult[d] += mult[j]
var tuples = [Z(mu[d]) for d in 1..=V]
var k = 1
loop:
    var count = Z(0)
    for d in 1..=V:
        tuples[d - 1] *= mult[d]
        count += tuples[d - 1]
    if count != Z(0): break
    k += 1
print(k - 1 + n - 1)
