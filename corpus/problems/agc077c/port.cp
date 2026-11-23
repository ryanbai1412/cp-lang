// Source: AtCoder AGC077 C (Reverse and DAG), https://atcoder.jp/contests/agc077/tasks/agc077_c
// Problem: a simple directed graph, n <= 2e5 vertices and m <= 2e5 edges with no pair joined
//   both ways, and 2 <= K <= n - 2. An operation picks K vertices and reverses every edge
//   inside them. Can some sequence make the graph acyclic? Multitest, sum n, m <= 2e5.
// Solution: complete the graph to a tournament, orienting each missing pair as we like.
//   An operation on S turns the out-degree o of v inside S into K - 1 - o, so it flips the
//   parity of every chosen vertex when K is even (parity is then no obstacle, and the editorial
//   shows the answer is always Yes) and keeps every
//   parity when K is odd. Then we need a completion with exactly floor(n/2) odd out-degrees,
//   as in the transitive one; start from low -> high on every pair, where vertex v has parity
//   (n - v) % 2, and toggle both ends of each given edge u > v. Reorienting a missing pair
//   toggles both ends, so a component of the missing-pair graph with sz vertices, odd of
//   them odd, reaches any count of odd's parity in [odd % 2, sz - (sz - odd) % 2]. The odd
//   count always has the parity of n(n-1)/2, which is that of floor(n/2), so summing the
//   intervals into [lo, hi] and testing floor(n/2) in it decides. The complement components
//   come from a BFS that keeps the unvisited vertices in a list and, at each vertex u,
//   splits off those not joined to u: each scan of a kept vertex is paid by an edge, O(n + m).

with io(answers: ("Yes", "No"))
@!multitest
let n: 4..=200_000 = read()
let m: 0..=200_000 = read()
let k: 2..=n - 2 = read()
domain V = 1..=n

let edges: [0..m](V, V) = read()
var odd_out: [V]Bool = [V] v => (n - 1 - v.ord) % 2 == 1
for (u, v) in edges if u > v:
    odd_out[u] = not odd_out[u]
    odd_out[v] = not odd_out[v]

if k % 2 == 0:
    print(yes(true))
    return

let joined = Graph.undirected(edges)

fn complement_components() -> List<List<V>>:
    var unvisited = V.collect()
    var near: [V]?V = None
    var comps: List<List<V>> = []
    while let s = unvisited.pop():
        var comp: List<V> = []
        var queue = Deque<V>()
        queue.push(s)
        while let Some(u) = queue.pop_front():
            comp.push(u)
            for v in joined[u]: near[v] = u
            let (apart, close) = unvisited.partition(v => near[v] != Some(u))
            for v in apart: queue.push(v)
            unvisited = close
        comps.push(comp)
    comps

var lo = 0
var hi = 0
for comp in complement_components():
    let sz = comp.len
    let odd = comp.count(v => odd_out[v])
    lo += odd % 2
    hi += sz - (sz - odd) % 2

let target = n / 2
print(yes(lo <= target <= hi))
