// Another Contest 7 P3 Network Connections: n computers with distinct
// values, m cables already laid. A new cable between i and j costs
// |a[i] - a[j]|. Print the least cost that connects every computer.
//
// Sorted by value, any cable between two computers can be replaced by the
// chain of neighbours between them for the same cost, so only the n - 1
// neighbouring pairs are candidate edges. Kruskal over those, after the
// free cables.
// original: corpus/submissions/3540475.cpp
use std.dsu.{Dsu}

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
    gaps.push((cost: a[v] - a[u], u: u, v: v))

var total = 0
for g in sorted(gaps, by: g => g.cost):
    if net.unite(g.u, g.v): total += g.cost
print(total)
