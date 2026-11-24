// Source: IATI 2021 Senior Day 1 Task 3 (News), https://dmoj.ca/problem/iati21a13
// Problem: a tree of N <= 2e5 workers rooted at 1, some initially knowing the news; Q <= 2e5
//   queries on x's subtree down to k levels below x (0 <= k <= N): "1 x k" tells all of them
//   the news, "2 x k" prints how many of them know it.
// Solution: a worker learns the news once, so first find offline the first update covering
//   each vertex: walking the preorder with a stack of open ancestors, a min SegTree indexed by
//   depth limit dep(x) + k holds the open updates (undone when x's subtree ends), and vertex u
//   takes the min over limits >= dep(u). Replaying the queries, a DynamicOrderTree over the
//   preorder holds each vertex's depth once it knows (n before), so a count is the number of
//   values <= dep(x) + k in x's subtree range, O((N + Q) log^2 N).
// Stdin: n, n-1 edges (1-based), init[0..n] (0/1), q, then q queries (ty, u, k).
// original: corpus/submissions/5052303.cpp

use std.segtree.{DynamicOrderTree}

let n: 1..=200_000 = read()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<[n - 1](V, V)>()), V.first!)
let knows: [V]0..=1 = read()
let q: 1..=200_000 = read()
domain Q = 0..q
let qs: [Q](ty: 1..=2, x: V, k: 0..=n) = read()

var ups: [V]List<Q> = [V] _ => []
for i in Q:
    if qs[i].ty == 1: ups[qs[i].x].push(i)

// first[v]: the first update that reaches v.
domain Lim = 0..n
var reach = SegTree(min, [Lim] _ => q)
var undo: List<(l: Lim, old: Int)> = []
var open: List<(v: V, mark: Int)> = []
var first: [V]?Q = None
for p in t.Pre:
    let v = t.at[p]
    while open.len > 0 and not t.is_anc(open.last!.v, v):
        let mark = open.pop()!.mark
        while undo.len > mark:
            let u = undo.pop()!
            reach.set(u.l, u.old)
    open.push((v:, mark: undo.len))
    for i in ups[v]:
        let l = Lim(min(t.depth(v) + qs[i].k, n - 1))
        undo.push((l:, old: reach.get(l)))
        reach.set(l, min(reach.get(l), i.ord))
    first[v] = Q.at(reach.fold(Lim(t.depth(v))..))

var wake: [Q]List<V> = [Q] _ => []
for v in V:
    if let f = first[v]:
        if knows[v] == 0: wake[f].push(v)

var told = DynamicOrderTree([t.Pre] p => if knows[t.at[p]] == 1: t.depth(t.at[p]) else: n)
for i in Q:
    let (ty, x, k) = qs[i]
    if ty == 1:
        for v in wake[i]: told.set(t.pos(v), t.depth(v))
    else:
        print(told.count_lt(t.sub(x), min(t.depth(x) + k, n - 1) + 1))
