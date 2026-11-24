// Source: Codeforces 2101B (Quartet Swapping), https://codeforces.com/contest/2101/problem/B
// limits: 2000 ms, 250 MB
// Problem: a permutation of n <= 2e5 (sum of n <= 2e5, multitest). An operation swaps a[i], a[i+1]
//   with a[i+2], a[i+3]. Print the lexicographically smallest reachable permutation.
// Solution: an operation is one transposition among even positions and one among odd ones, so the
//   two classes keep their values and the xor of their inversion parities is invariant, and every
//   arrangement keeping both is reachable. Sort each class; if the parities (Fenwick tree
//   inversion counts) differ, swap positions n - 3 and n - 1, the last two of one class.
//   O(n log n).
// Bounds: n: 4..=200_000; a: 1..=n

@!multitest
let n: 4..=200_000 = read()
domain V = 1..=n
var a: [0..n]V = read()

fn parity(vals: List<V>) -> Int:
    var bit = Fenwick<V, Int>()
    var ans = 0
    for (seen, v) in vals.items:
        ans ^= (seen - bit.prefix(v.r)) & 1
        bit.add(v, 1)
    ans

var x = [a[i] for i in 0..n by 2]
var y = [a[i] for i in 1..n by 2]
let mismatch = (parity(x) ^ parity(y)) != 0
x.sort()
y.sort()
for (i, v) in x.items: a[2 * i] = v
for (i, v) in y.items: a[2 * i + 1] = v
if mismatch: a.swap(n - 3, n - 1)
print(a)
