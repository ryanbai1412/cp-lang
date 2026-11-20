// IATI '21 A13: dynamic subtree depth-bounded activation and counting queries on a tree.
// Rooted<V> handles DFS preorder ranges and depths; a SegTree over SortedBag<Int> finds
// each node's earliest activating update during Euler DFS; a SegTree of ordered Set<(Int, Int)>
// answers dynamic 2D subtree-depth range counts via Set.rank (replacing __gnu_pbds::tree).
// original: corpus/submissions/5052303.cpp

use std.algebra.{Monoid}
use std.bag.{SortedBag}
use std.fenwick.{Fenwick}

let n: Int = read()
domain V = 0..n
var edges: List<(V, V)> = []
for _ in 0..n - 1:
    let (u, v): (1..=n, 1..=n) = read()
    edges.push((V(u - 1), V(v - 1)))

let init_on: [V]0..=1 = read()
let q: Int = read()
type Query = (ty: Int, u: V, k: Int)
let queries: List<Query> = [
    (ty: read::<Int>(), u: V(read::<Int>() - 1), k: read::<Int>())
    for _ in 0..q
]

let g = Graph.undirected(edges)
let tree = Rooted(g, V.first)

var upds_at: [V]List<(Int, Int)> = []
for (qi, qry) in queries.items:
    if qry.ty == 1: upds_at[qry.u].push((qry.k, qi))

// Find earliest update activating each node u via DFS + depth-indexed SegTree of SortedBags.
domain Depth = 0..=(n + 2)
var bags: [Depth]SortedBag<Int> = SortedBag()
var st = SegTree<Depth, ?Int>(Monoid(none, min), [Depth] _ => none)
var activate_at: [0..q]List<V> = []

fn dfs(u: V):
    let d = tree.depth(u)
    for (k, qi) in upds_at[u]:
        let idx = Depth(min(n + 1, d + k))
        bags[idx].add(qi)
        st.set(idx, bags[idx].first)
    if let first_q = st.fold(Depth(d)..): activate_at[first_q].push(u)
    for v in tree.children(u): dfs(v)
    for (k, qi) in upds_at[u]:
        let idx = Depth(min(n + 1, d + k))
        bags[idx].remove(qi)
        st.set(idx, bags[idx].first)

dfs(V.first)

// Fast Fenwick path when every query has k == n, else SegTree of ordered Sets with .rank().
let all_full = all(qry.k == n for qry in queries)
domain Pre = 0..n
if all_full:
    var bit: Fenwick<Pre, Int> = Fenwick()
    var active: [Pre]Bool = false
    for u in V if init_on[u] == 1:
        active[Pre(tree.pos(u).idx)] = true
        bit.add(Pre(tree.pos(u).idx), 1)
    for (qi, qry) in queries.items:
        if qry.ty == 1:
            for u in activate_at[qi]:
                let p = Pre(tree.pos(u).idx)
                if not active[p]:
                    active[p] = true
                    bit.add(p, 1)
        else:
            let r = tree.sub(qry.u)
            print(bit.sum(Pre(r.lo.idx)..=Pre(r.hi.idx)))
else:
    var cur_val: [Pre]?Int = none
    for u in V if init_on[u] == 1: cur_val[Pre(tree.pos(u).idx)] = tree.depth(u)
    var seg: List<Set<(Int, Int)>> = List.filled(4 * n + 4, Set())

    fn seg_add(o: Int, l: Int, r: Int, idx: Int, val: Int):
        seg[o].add((val, idx))
        if l == r: return
        let m = (l + r) / 2
        if idx <= m: seg_add(o * 2, l, m, idx, val)
        else: seg_add(o * 2 + 1, m + 1, r, idx, val)

    fn seg_count(o: Int, l: Int, r: Int, ql: Int, qr: Int, lim: Int) -> Int:
        if ql <= l and r <= qr: return seg[o].rank((lim + 1, -1))
        let m = (l + r) / 2
        var ans = 0
        if ql <= m: ans += seg_count(o * 2, l, m, ql, qr, lim)
        if m < qr: ans += seg_count(o * 2 + 1, m + 1, r, ql, qr, lim)
        ans

    for u in V if init_on[u] == 1:
        seg_add(1, 0, n - 1, tree.pos(u).to_idx, tree.depth(u))

    for (qi, qry) in queries.items:
        if qry.ty == 1:
            for u in activate_at[qi]:
                let p = tree.pos(u)
                if cur_val[p] == none:
                    cur_val[p] = tree.depth(u)
                    seg_add(1, 0, n - 1, p.to_idx, tree.depth(u))
        else:
            let r = tree.sub(qry.u)
            print(seg_count(1, 0, n - 1, r.first!.to_idx, r.last!.to_idx, tree.depth(qry.u) + qry.k))
