// Source: CCO 2003 P4 (Constrained Permutations), https://dmoj.ca/problem/cco03p4
// limits: 1000 ms, 16 MB
// Problem: n <= 9 and k constraints "x comes before y". Print how many permutations of 1..=n
//   satisfy them all.
// Solution: place the numbers front to back; need[y] is the Subset of numbers that must come
//   before y. ways[s] counts orders of the set s placed first; y can join s once need[y] is
//   inside s.
//   O(2^n n).
// original: corpus/submissions/5024536.cpp
// Bounds: n: 1..=9; rules.x, rules.y: 1..=n
let n: 1..=9 = read()
let k: Int = read()
domain V = 1..=n
type Placed = Subset<V>
let rules: [k](x: V, y: V) = read()  // x comes before y

var need: [V]Placed = Placed.empty
for r in rules if r.x != r.y: need[r.y] = need[r.y].with(r.x)

var ways: [Placed]Int = 0
ways[Placed.empty] = 1
for s in Placed:
    for v in V if not s.contains(v) and (need[v] - s).is_empty: ways[s.with(v)] += ways[s]
print(ways[Placed.full])
