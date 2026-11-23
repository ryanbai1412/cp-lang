// Source: CCO 2017 Day 1 P1 (Vera and Trail Building), https://dmoj.ca/problem/cco17p1
// Problem: given K <= 1e7, print a connected multigraph with V, E <= 5000 that has exactly K
//   pairs a < b joined by a round trip a -> b -> a using no trail twice.
// Solution: such pairs are the pairs inside one 2-edge-connected component, so a cycle on c
//   vertices gives c(c - 1) / 2 of them and bridges none. Greedily take the largest c with
//   c(c - 1) / 2 <= what is left (c = 2 is a double edge) and chain the cycles by bridges;
//   at most 4587 vertices for any K. O(V).
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
