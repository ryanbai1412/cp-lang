// Baltic OI 2020 Joker: given an undirected graph with n vertices and m edges,
// answer q queries asking whether deleting edges in [l, r] leaves an odd cycle
// (i.e. a non-bipartite graph).
//
// For each left endpoint l, the smallest right endpoint `ans[l]` such that
// keeping edges outside `[l, r]` is still non-bipartite is monotone in l.
// Divide-and-conquer computes `ans[0..m]` in O((m + q) log^2 n) using
// `std.dsu.ParityDsu`, checkpointing with `mark` and `undo_to`.
// original: corpus/submissions/4656866.cpp
use std.dsu.{ParityDsu}

let (n, m, q): (Int, Int, Int) = read()
domain V = 1..=n
domain E = 0..m

let edges: [E](u: V, v: V) = read()

// Fast path: if the full graph is already bipartite, no subgraph has an odd cycle.
var full = ParityDsu<V>()
for e in E: full.unite(edges[e].u, edges[e].v)

if full.consistent:
    for _ in 0..q:
        let (_, _): (Int, Int) = read()
        print("NO")
else:
    var dsu = ParityDsu<V>()
    var max_r: [E]Int = -1

    fn add(idx: Int):
        let e = edges[E(idx)]
        dsu.unite(e.u, e.v)

    // Invariant on entry: edges `[0..l)` and `(qr..m)` are already in `dsu`.
    fn solve_dnc(l: Int, r: Int, ql: Int, qr: Int):
        if l > r: return
        let mid = (l + r) / 2
        let before_left = dsu.mark()
        for i in l..mid: add(i)

        let before_right = dsu.mark()
        var split = qr
        while split >= ql and dsu.consistent:
            add(split)
            split -= 1
        max_r[E(mid)] = split
        dsu.undo_to(before_right)

        add(mid)
        solve_dnc(mid + 1, r, split, qr)
        dsu.undo_to(before_left)

        for i in split + 1..=qr: add(i)
        solve_dnc(l, mid - 1, ql, split)
        dsu.undo_to(before_left)

    solve_dnc(0, m - 1, 0, m - 1)
    for _ in 0..q:
        let (l, r): (Int, Int) = read()
        if r - 1 <= max_r[E(l - 1)]: print("YES")
        else: print("NO")
