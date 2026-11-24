// Source: IOI 2014 Day 1 P3, Game, https://dmoj.ca/problem/ioi14p3
// limits: 2500 ms, 256 MB
// Problem: interactive: n cities (4 <= n <= 1500); every pair (u, v) is asked once, in an
//   order you do not know, and hasEdge(u, v) must answer 1 or 0 at once. Answer so that
//   whether the graph is connected stays undetermined until the last of the n(n-1)/2 pairs.
// Solution: Say yes to a pair only when it is the last unasked pair between the two current
//   components, then merge them; so components join only when forced and the graph becomes
//   connected exactly on the last question. A std Dsu keeps the components; asked[Pair(a, b)]
//   counts asked pairs between roots a and b and folds into the kept root's pairs in O(n)
//   per merge, so O(n^2) in total.
// Reversed callback interactive, stdio-adapted today-form: print READY, then read each pair
// and print the previous reply.
// original: corpus/submissions/4683972.cpp
// Bounds: n: 2..=1500
let n: 2..=1500 = read()
domain V = 0..n

var dsu = Dsu<V>()
var asked: [Pair<V>]Int = 0

// yes only to the last unasked pair between two components
fn answer(u0: V, v0: V) -> Int:
    let (u, v) = (dsu.find(u0), dsu.find(v0))
    if u == v: return 0
    asked[u, v] += 1
    if asked[u, v] < dsu.size(u) * dsu.size(v): return 0
    let (kept, merged) = dsu.merge(u, v)!
    for i in V if i != kept and i != merged:
        asked[kept, i] += asked[merged, i]
    1

var pair: (V, V) = ask("READY")
for _ in 1..n * (n - 1) / 2:
    pair = ask(answer(pair.0, pair.1))
print(answer(pair.0, pair.1))
