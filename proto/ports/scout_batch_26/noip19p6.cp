// NOIP '19 P6 - Centroid of Tree (30pt): for every edge of a tree, cut it and
// add up the labels of the centroids of the two parts (a part can have two).
// Print the total, for each of several trees.
// Root the tree at a centroid. Any other vertex x keeps every child subtree
// at or below half of whatever part it lands in, so x is a centroid of its
// part exactly when the part that was cut away has between n - 2*size(x)
// and n - 2*heavy(x) vertices. Cutting above x removes n - size(a) for an
// ancestor a; cutting elsewhere (never inside x's subtree, where x is too
// light) removes size(v). Three counters over "vertices removed" answer it
// in one walk. The root is a centroid unless the cut leaves its heaviest
// remaining child above half.
// original: corpus/submissions/5148136.cpp
use std.graph.{Graph}
use std.tree.{Rooted}
use std.fenwick.{Fenwick}
use std.algebra.{Sum}

let tests: 1..=5 = read()
for _ in 0..tests:
    let n: 1..=299995 = read()
    domain V = 1..=n
    domain E = 0..(n - 1)
    domain Size = 0..=n
    let edges: [E](u: V, v: V) = read()
    let g = Graph<V, E>.undirected(edges)

    // WISH TR6: `let t = Rooted<V>.at_centroid(g)` (or `t.centroid()` and
    // `t.reroot(c)`); today the tree is rooted twice, the first time only
    // to learn the sizes that locate the centroid.
    let first = Rooted<V>.new(u => g.adj(u), V.first)
    var c = V.first
    while let h = first.heavy[c]:
        if 2 * first.size[h] <= n: break
        c = h
    let t = Rooted<V>.new(u => g.adj(u), c)

    fn heavy(x: V) -> Int:
        if let h = t.heavy[x]: t.size[h]
        else: 0

    var total = 0

    // the root: its two largest children decide
    var second = 0
    for k in t.kids[c]:
        if some(k) != t.heavy[c]: second = max(second, t.size[k])
    for v in V:
        if v == c: continue
        let rest = if t.is_anc(t.heavy[c]!, v): second else: heavy(c)
        if 2 * rest <= n - t.size[v]: total += c.label

    // every other vertex
    var all = Fenwick<Size, Int>.new(Sum<Int>)      // size(v) over all v != root
    var seen = Fenwick<Size, Int>.new(Sum<Int>)     // size(v) over v visited so far
    var chain = Fenwick<Size, Int>.new(Sum<Int>)    // size(a) over proper ancestors a
    var above = Fenwick<Size, Int>.new(Sum<Int>)    // n - size(a) over ancestors a, self included
    for v in V:
        if v != c: all.add(Size(t.size[v]), 1)

    // WISH TR7: an enter/exit walk, `for (v, entering) in t.tour:`, so the
    // "ancestors of the current vertex" multiset needs no recursion; a
    // 3e5-vertex path overflows the judge's stack in this form.
    fn walk(x: V):
        let want = Size(max(n - 2 * t.size[x], 0))..=Size(n - 2 * heavy(x))
        above.add(Size(n - t.size[x]), 1)
        let before = seen.fold(want)
        seen.add(Size(t.size[x]), 1)
        let up = chain.fold(want)
        chain.add(Size(t.size[x]), 1)
        for k in t.kids[x]: walk(k)
        chain.add(Size(t.size[x]), -1)
        let below = seen.fold(want) - before
        total += x.label * (all.fold(want) - below - up + above.fold(want))
        above.add(Size(n - t.size[x]), -1)

    for k in t.kids[c]: walk(k)
    print(total)
