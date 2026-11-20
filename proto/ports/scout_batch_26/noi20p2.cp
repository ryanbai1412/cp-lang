// NOI '20 P2 - Destiny (25pt): colour every edge of a tree rooted at 1 with 0
// or 1 so that each of m given ancestor-descendant paths (u above v) holds
// at least one 1. Count the colourings mod 998244353.
// For a vertex x, g[x][d] counts the colourings of the edges below x in
// which every path that still has no 1 starts at depth d or higher (depth
// 1 = the root; d = 0 means "nothing pending"). An edge x-c coloured 1
// settles everything pending below c, coloured 0 passes it up, so
//     g[x][d] = [d >= need[x]] * prod over children c of (g[c][d] + g[c][depth x]).
// The row of x is the row of its heavy child, edited in place; the light
// children are multiplied in.
// original: corpus/submissions/5655455.cpp
use std.graph.{Graph}
use std.tree.{Rooted}
use std.mod.{Mod}

type Z = Mod<998244353>

let n: 1..=5e5 = read()
domain V = 1..=n
domain E = 0..(n - 1)
let edges: [E](u: V, v: V) = read()
let m: 1..=5e5 = read()
domain Path = 0..m
let paths: [Path](u: V, v: V) = read()

let g0 = Graph<V, E>.undirected(edges)
let t = Rooted(u => g0.adj(u), V.first)

// need[v]: the deepest start (as depth + 1) among the paths that end at v
var need: [V]Int = 0
for p in paths: need[p.v] = max(need[p.v], t.depth(p.u) + 1)

// WISH TR10: this row is a dense `List<Z>` of length depth(x) + 1, so a
// path-shaped tree costs O(n^2). The accepted solution keeps each row as a
// sparse segment tree over depths with a lazy multiplier and *merges* two
// trees node by node. The language has no mergeable structure:
//     var row: SparseSeg<Depth, Z, Mul> = take(g[heavy])
//     for c in light: row.merge(take(g[c]), (a, b) => a * b)
// needs both an owning `take` out of `[V]T` and a library tree whose nodes
// can change owner.
var g: [V]List<Z> = [V] _ => []
for x in t.preorder.rev:
    let d = t.depth(x) + 1
    // WISH TR11: `var row = take(g[h])` to move the heavy child's row up
    // without a copy; the interpreter aliases here, the compiler would reject it.
    var row: List<Z> = []
    if let h = t.heavy[x]:
        row = g[h]
        g[h] = []
        let all = row[d]
        row.truncate(d + 1)
        for i in 0..=d: row[i] = row[i] + all
        for c in t.children(x):
            if c == h: continue
            for i in 0..=d: row[i] = row[i] * (g[c][i] + g[c][d])
            g[c] = []
    else:
        for i in 0..=d: row.push(Z(1))
    for i in 0..need[x]: row[i] = Z(0)
    g[x] = row
print(g[V.first][0])
