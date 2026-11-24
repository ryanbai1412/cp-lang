// Source: Another Contest 7 P3 (Network Connections), https://dmoj.ca/problem/acc7p3
// Problem: n <= 1e5 users with distinct values f_i <= 1e9 (increasing with i) and m <= 1e5
//   existing friendships. Making i and j friends costs |f_i - f_j|; print the least cost that
//   connects everyone.
// Solution: sorted by value, a link between any two users can be replaced by the chain of
//   neighbouring pairs between them at the same total cost, so only the n - 1 neighbouring pairs
//   are candidates. Unite the existing friendships in a DSU, then Kruskal over those pairs by
//   cost. O(n log n + m).
// original: corpus/submissions/3540475.cpp

let n: Int = read()
let m: Int = read()
domain C = 0..n
let a: [C]Int = read()
var net = Dsu<C>.new()
for _ in 0..m:
    let u: Int = read()
    let v: Int = read()
    net.unite(C(u - 1), C(v - 1))

let by_value = sorted(C, by: c => a[c])
var gaps: List<(cost: Int, u: C, v: C)> = []
for i in 1..n:
    let (u, v) = (by_value[i - 1], by_value[i])
    gaps.push((cost: a[v] - a[u], u:, v:))

var total = 0
for g in gaps.sorted(by: g => g.cost):
    if net.unite(g.u, g.v): total += g.cost
print(total)
