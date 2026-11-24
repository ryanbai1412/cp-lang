// Source: DMOPC 2021 Contest 2 P5 (Permutations), https://dmoj.ca/problem/dmopc21c2p5
// limits: 2000 ms, 256 MB
// Problem: a tree on n <= 3000 nodes, node i holding value i. An operation swaps the values at
//   the ends of an edge and deletes the edge. Print how many permutations of the values can
//   result from exactly k < n operations, mod 998244352.
// Solution: a set S of k used edges gives prod over nodes of deg_S(v)! permutations (only the
//   order of the used edges at each node matters), so sum that over all |S| = k. Tree knapsack:
//   ways[d][i] counts the edge sets of v's subtree with i edges, d of them at v; weighting by
//   d! (parent edge unused) or (d + 1)! (used) gives v's two answers for its parent. Since
//   998244352 = 2^23 * 7 * 17 divides 26!, d stops at 26: O(26 n^2).
// original: corpus/submissions/3968384.cpp
// Bounds: n: 1..=3000; k: 0..=n - 1; edges.u, edges.v: 1..=n

const MOD = 998_244_352
type Z = Mod<MOD>
// 26! is 0 mod 998244352 = 2^23 * 7 * 17, so a node using 26 or more edges adds nothing
domain Deg = 0..=26

let n: 1..=3000 = read()
let k: 0..n = read()
domain Node = 1..=n
let edges: [n - 1](u: Node, v: Node) = read()
let tree = Rooted(Graph.undirected(edges), Node.first!)

var fact: [Deg]Z = 1
for d in Deg:
    if let up = d.next: fact[up] = fact[d] * Z(up.ord)

// by edges used in the subtree: the parent edge unused (`free`) or used and counted (`tied`)
var free: [Node]List<Z> = []
var tied: [Node]List<Z> = []
for u in tree.at.rev:
    let size = tree.size(u)
    var ways: [Deg]List<Z> = [Deg] _ => [Z(0) for _ in 0..size]
    ways[Deg.first!][0] = 1
    var seen = 0
    for c in tree.children(u):
        var next: [Deg]List<Z> = [Deg] _ => [Z(0) for _ in 0..size]
        for d in Deg:
            for i in 0..=seen if ways[d][i] != 0:
                for j in 0..free[c].len: next[d][i + j] += ways[d][i] * free[c][j]
                if let up = d.next:
                    for j in 1..tied[c].len: next[up][i + j] += ways[d][i] * tied[c][j]
        seen += tree.size(c)
        swap(mut ways, mut next)
    free[u] = [sum(ways[d][i] * fact[d] for d in Deg) for i in 0..size]
    tied[u] = [Z(0)] + [sum(ways[d][i] * fact[d] * Z(d.ord + 1) for d in Deg) for i in 0..size]

print(free[tree.root][k])
