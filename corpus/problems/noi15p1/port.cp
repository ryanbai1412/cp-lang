// Source: NOI 2015 P1 (Automated Program Analyzer), https://dmoj.ca/problem/noi15p1
// limits: 2000 ms, 512 MB
// Problem: t independent instances, each n <= 1e5 constraints x_i = x_j or x_i != x_j with
//   indices up to 1e9. For each, print YES if all its constraints can hold at once, else NO.
// Solution: compress the indices, unite the two sides of every equality in a Dsu, then check
//   that no inequality has both sides in one class. O(n log n) per instance.
// original: corpus/submissions/3775616.cpp
// Bounds: q: 1..=1e5; cons.i, cons.j: 1..=1e9; cons.e: 0..=1

@multitest
let q: 1..=1e5 = read()
let cons = read<[q](i: 1..=1e9, j: 1..=1e9, e: 0..=1)>()
let (domain X, ids) = compress(cons.flat_map(c => [c.i, c.j]))
let at = (v: Int) => ids.index_of(v)
var eq = Dsu<X>()
for c in cons if c.e == 1: eq.unite(at(c.i), at(c.j))
print(
    yes(cons.filter(c => c.e == 0).all(c => not eq.same(at(c.i), at(c.j))), answers: ("YES", "NO"))
)
