// Source: AtCoder ABC481 F (Evil Shortest Path), https://atcoder.jp/contests/abc481/tasks/abc481_f
// Problem: a directed graph, N, M <= 2e5, each edge with a digit 1..9, and every vertex
//   reachable from 1. For each v = 2..N, the smallest number spelled by the digits of a walk
//   from 1 to v, modulo 998244353.
// Solution: with nonzero digits, fewer edges is smaller, so take BFS layers, and within a
//   layer the lexicographically smallest digit string. Keep each layer as an ordered list
//   of groups of vertices sharing the same smallest string; a group sends its unvisited
//   out-neighbours to buckets by digit 1..9, and the buckets in order form the next layer's
//   groups. A vertex's value is its group's value * 10 + digit: O(N + M).
// Bounds: n: 2..=2e5; m: 1..=2e5; edges.u, edges.v: 1..=n; edges.w: 1..=9

type Z = Mod<998_244_353>

let (n, m): (2..=2e5, 1..=2e5) = read()
domain V = 1..=n
let edges: [m](u: V, v: V, w: 1..=9) = read()
let g = Graph.directed(edges)

// value[v] is set once, by the first group (in layer order) that reaches v with the least digit
var value: [V]?Z = None
value[V.first!] = Z(0)
var layer: List<List<V>> = [[V.first!]]
while not layer.is_empty:
    var next: List<List<V>> = []
    for group in layer:
        let here = value[group[0]]!
        var by_digit: [1..=9]List<V> = []
        for u in group:
            for a in g.out(u) if value[a.to].is_none:
                by_digit[a.w].push(a.to)
        for (d, reached) in by_digit.items:
            var fresh: List<V> = []
            for v in reached if value[v].is_none:
                value[v] = here * 10 + d
                fresh.push(v)
            if not fresh.is_empty: next.push(fresh)
    layer = next

print([value[v]! for v in V if v != V.first!], sep: "\n")
