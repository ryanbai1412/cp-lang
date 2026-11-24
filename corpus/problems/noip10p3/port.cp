// Source: NOIP 2010 Senior P3 (Imprisoning Criminals), Luogu P1525 (no longer on DMOJ)
// Problem: N <= 2e4 criminals, M <= 1e5 distinct hostile pairs (a, b) with anger c <= 1e9.
//   Split them over two prisons to minimise the largest anger of a pair in the same prison;
//   print that value, or 0 if no pair has to share.
// Solution: greedy from the angriest feud down, forcing each pair into different prisons. A
//   DSU groups criminals known to share a prison, and foe[x] remembers one criminal known to
//   be in the other one: separating a and b unites b with foe[a] and a with foe[b]. The first
//   feud whose pair is already united is the answer. O(M log M) for the sort.
// original: corpus/submissions/5205348.cpp
// Bounds: n: 2..=20000; m: 1..=100000; feuds.a, feuds.b: 1..=n; feuds.anger: 1..=1e9
let (n, m): (2..=20000, 1..=100000) = read()
domain V = 1..=n
let feuds: [0..m](a: V, b: V, anger: 1..=1e9) = read()

var together = Dsu<V>()
var foe: [V]?V = None  // someone known to be in the other prison
for (a, b, anger) in feuds.sorted(by: f => desc(f.anger)):
    if together.same(a, b):
        print(anger)
        return
    if let f = foe[a]: together.unite(f, b) else: foe[a] = b
    if let f = foe[b]: together.unite(f, a) else: foe[b] = a
print(0)
