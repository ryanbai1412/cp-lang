// cp: 0.1 1f59ac0+dirty
// CCO 2017 Vera and Trail Building: build a graph with exactly k "beautiful"
// pairs (two vertices joined by two edge-disjoint paths). A cycle of c
// vertices has c (c - 1) / 2 such pairs and bridges add none, so chain
// greedily chosen cycles together with bridges.
let k: 1..=1e7 = read()

fn pairs(c: Int) -> Int: c * (c - 1) / 2

var cycles: List<Int> = []
var left = k
var c = 5000
while left > 0:
    while pairs(c) > left: c -= 1
    left -= pairs(c)
    cycles.push(c)

var trails: List<(u: Int, v: Int)> = []
var placed = 0
for c in cycles:
    let (first, last) = (placed + 1, placed + c)
    if placed > 0: trails.push((placed, first))
    trails.push((first, last))
    for v in first..last: trails.push((v, v + 1))
    placed = last
print(placed, trails.len)
for (u, v) in trails: print(u, v)
