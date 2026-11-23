// Source: AtCoder AGC077 C (Reverse and DAG), https://atcoder.jp/contests/agc077/tasks/agc077_c
// Problem: a simple directed graph, n <= 2e5 vertices and m <= 2e5 edges with no pair joined
//   both ways, and 2 <= K <= n - 2. An operation picks K vertices and reverses every edge
//   inside them. Can some sequence make the graph acyclic? Multitest, sum n, m <= 2e5.
// Solution: the answer is always Yes for even K. For odd K, complete the graph to a
//   tournament (missing pairs oriented low -> high); an operation then keeps every out-degree
//   parity, so we need a completion with floor(n/2) odd out-degrees, as in the transitive
//   tournament. Orienting a missing pair the other way toggles both ends, so a component of
//   the complement graph with sz vertices, odd of them odd, can reach any count of odd's
//   parity in [odd % 2, sz - (sz - odd) % 2]; sum the intervals and test floor(n/2). The
//   complement components come from a DFS over a sorted set of unvisited vertices,
//   O((n + m) log n).

@!multitest
let n: 1..=200_000 = read()
let m: 0..=200_000 = read()
let k: 1..=1_000_000_000 = read()
domain V = 1..=n

let edges: [0..m](V, V) = read()
var odd_out: [V]Bool = [V] v => (n - 1 - v.ord) % 2 == 1
for (u, v) in edges if u > v:
    odd_out[u] = not odd_out[u]
    odd_out[v] = not odd_out[v]

if k % 2 == 0:
    print("Yes")
    return

let g = Graph.undirected(edges)
let forbidden = [V] u => g[u].sorted()

fn complement_components() -> List<List<V>>:
    var unvisited = Set(V)
    var comps: List<List<V>> = []
    for start in V if start in unvisited:
        var comp: List<V> = []
        fn dfs(cur: V):
            unvisited.remove(cur)
            comp.push(cur)
            var cursor: ?V = V.first
            var ai = 0
            while let at = cursor:
                if let nxt = unvisited.first_ge(at):
                    while ai < forbidden[cur].len and forbidden[cur][ai] < nxt: ai += 1
                    if ai == forbidden[cur].len or forbidden[cur][ai] != nxt: dfs(nxt)
                    cursor = nxt.next
                else:
                    break
        dfs(start)
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
print(if lo <= target and target <= hi: "Yes" else: "No")
