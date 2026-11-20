// MMCC '14 P3 - Esdeath (20pt): put k outposts on the vertices of a tree so
// that the farthest vertex from its nearest outpost is as close as possible;
// print that distance.
// Fix a radius and place greedily from the leaves up: each vertex reports
// the farthest descendant still uncovered and the nearest outpost below it,
// and an outpost goes on a child only when its uncovered descendant could
// not survive one more edge. The outposts needed fall as the radius grows,
// so the least radius that needs at most k is found by bisection.
// original: corpus/submissions/5120083.cpp
use std.graph.{Graph}
use std.tree.{Rooted}

let (n, k): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
domain E = 0..(n - 1)
let edges: [E](u: V, v: V) = read()
let g = Graph<V, E>.undirected(edges)
let t = Rooted(u => g.adj(u), V.first)

fn outposts(radius: Int) -> Int:
    var open: [V]?Int = [V] _ => none      // farthest uncovered vertex below, if any
    var post: [V]?Int = [V] _ => none      // nearest outpost below, if any
    var used = 0
    for v in t.preorder.rev:
        var far = 0
        var near: ?Int = none
        for c in t.children(v):
            if let d = post[c]: near = min(near ?? n, d + 1)
            if let d = open[c]:
                if d + 1 > radius:
                    used += 1
                    near = 1
                else:
                    far = max(far, d + 1)
        post[v] = near
        open[v] = if far + (near ?? n) <= radius: none else: far
    if open[V.first] != none: used += 1
    used

// WISH TR4: `bsearch_first(0..n, r => outposts(r) <= k)`
var (lo, hi) = (0, n)
while lo < hi:
    let mid = (lo + hi) / 2
    if outposts(mid) <= k: hi = mid
    else: lo = mid + 1
print(lo)
