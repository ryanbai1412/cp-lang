// AtCoder Grand Contest 077 C: decide whether an orientation reachable by
// flipping non-forbidden pairs can give the target out-degree parity profile.
//
// Start from the transitive tournament (i -> j for i < j), where vertex i has
// out-degree parity (n - 1 - i) mod 2, and flip both endpoints for each given
// backward edge (u > v).
// - When k is even, every allowed k-step parity balance reduces to a condition
//   satisfied by every tournament, so the answer is always "Yes".
// - When k is odd, we need exactly `target = floor(n / 2)` vertices with odd
//   out-degree. Because the sum of all out-degrees in any tournament is
//   `n * (n - 1) / 2`, and `n * (n - 1) / 2 ≡ floor(n / 2) (mod 2)` for every
//   n (both n = 2t and n = 2t + 1 give parity t mod 2), the total number of
//   odd-out-degree vertices ALWAYS has the same parity as `target`.
// Each allowed flip toggles two endpoints of an edge in the complement graph,
// so inside a complement component of size sz with `odd` initially-odd vertices
// we can achieve any odd count in [odd % 2, sz - (sz - odd) % 2] of the same
// parity as `odd`. Summing these intervals gives [lo, hi] with the same parity
// as `target`, so `lo <= target <= hi` is both necessary and sufficient.

use std.graph.undirected_of_pairs

@!multitest
let n: 1..=200_000 = read()
let m: 0..=200_000 = read()
let k: 1..=1_000_000_000 = read()
domain V = 1..=n

let edges: [0..m](V, V) = read()
var odd_out: [V]Bool = [V] v => (n - 1 - v.ord) % 2 == 1
for (u, v) in edges if u > v:
    odd_out[u] = not odd_out[u]
    odd_out[v] = not odd_out[v]

if k % 2 == 0:
    print("Yes")
    return

let forbidden = undirected_of_pairs(List(edges))

var lo = 0
var hi = 0
let (domain C, comp) = components(complement(forbidden))
for group in comp.buckets():
    let sz = group.len
    let odd = group.count(v => odd_out[v])
    lo += odd % 2
    hi += sz - (sz - odd) % 2

let target = n / 2
print(if lo <= target <= hi: "Yes" else: "No")
