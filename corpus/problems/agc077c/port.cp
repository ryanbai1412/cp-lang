// Source: AtCoder AGC077 C (Reverse and DAG), https://atcoder.jp/contests/agc077/tasks/agc077_c
// limits: 2000 ms, 1024 MB
// Problem: a simple directed graph, n <= 2e5 vertices and m <= 2e5 edges with no pair joined
//   both ways, and 2 <= K <= n - 2. An operation picks K vertices and reverses every edge
//   inside them. Can some sequence make the graph acyclic? Multitest, sum n, m <= 2e5.
// Solution: complete the graph to a tournament, orienting each missing pair as we like; the
//   out-degree parities are what decides it. An operation on S turns the out-degree o of v
//   inside S into K - 1 - o, so it flips the parity of every chosen vertex when K is even and
//   keeps every parity when K is odd. A transitive tournament has floor(n/2) odd out-degrees,
//   and the odd count always has the parity of n(n-1)/2, which is that of floor(n/2).
//   Even K: since K + 1 <= n, the operations on S + a and S + b (|S| = K - 1) together flip
//   just a and b, so any even set of parities can be flipped and the answer is always Yes.
//   Odd K: we need a completion with exactly floor(n/2) odd out-degrees. Start from
//   low -> high on every pair, where vertex v has parity (n - v) % 2, and toggle both ends of
//   each given edge u > v. Reorienting a missing pair toggles both ends, so a component of
//   the missing-pair graph with sz vertices, odd of them odd, reaches any count of odd's
//   parity in [odd % 2, sz - (sz - odd) % 2]; sum the intervals into [lo, hi] and test
//   floor(n/2), whose parity is right. `components(complement(...))` walks the unvisited
//   vertices only, O(n + m).
// Bounds: n: 4..=200_000; m: 0..=200_000; k: 2..=n - 2

use std.graph.{complement, components}

with io(answers: ("Yes", "No"))
@!multitest
let n: 4..=200_000 = read()
let m: 0..=200_000 = read()
let k: 2..=n - 2 = read()
domain V = 1..=n

let edges: [0..m](V, V) = read()
if k % 2 == 0:
    print(yes(true))
    return

var odd_out: [V]Bool = [V] v => (n - 1 - v.ord) % 2 == 1
for (u, v) in edges if u > v:
    odd_out[u] = not odd_out[u]
    odd_out[v] = not odd_out[v]

let (domain C, comp) = components(complement(Graph.undirected(edges)))
var size: [C]Int = 0
var odd: [C]Int = 0
for v in V:
    size[comp[v]] += 1
    if odd_out[v]: odd[comp[v]] += 1

let lo = sum(odd[c] % 2 for c in C)
let hi = sum(size[c] - (size[c] - odd[c]) % 2 for c in C)
let target = n / 2
print(yes(lo <= target <= hi))
