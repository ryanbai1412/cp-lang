// Source: CSP-S 2019 Day 2 T3 (Centroid of Tree, DMOJ noip19p6; the DMOJ page is not public),
//   https://www.luogu.com.cn/problem/P5666
// Problem: T <= 5 trees of n <= 299995 vertices. A centroid leaves no part above floor(size / 2)
//   when removed. For each tree print the sum, over every edge, of the labels of all centroids
//   (one or two) of both halves left by cutting that edge.
// Solution: root at a centroid c. Another vertex x keeps its children at most half of its part,
//   so it is a centroid exactly when the cut-away part has between n - 2 size(x) and
//   n - 2 heavy(x) vertices. Cutting above x removes n - size(a) for an ancestor a, cutting
//   elsewhere outside x's subtree removes size(v): Fenwick counters over "vertices removed"
//   (all, seen, chain, above) answer every x in one DFS. c counts unless the cut leaves its
//   heaviest remaining child above half. O(n log n) per tree.
// original: corpus/submissions/5148136.cpp

let tests: 1..=5 = read()
for _ in 0..tests:
    let n: 1..=299995 = read()
    domain V = 1..=n
    domain E = 0..(n - 1)
    domain Size = 0..=n
    let edges: [E](u: V, v: V) = read()
    let g = Graph<V, E>.undirected(edges)

    let c = Rooted.new(g, V.first!).centroid()
    let t = Rooted.new(g, c)

    fn heavy(x: V) -> Int:
        if let h = t.heavy[x]: t.size(h)
        else: 0

    var total = 0

    // the root: its two largest children decide
    var second = 0
    for k in t.children(c):
        if Some(k) != t.heavy[c]: second = max(second, t.size(k))
    for v in V:
        if v == c: continue
        let rest = if t.is_anc(t.heavy[c]!, v): second else: heavy(c)
        if 2 * rest <= n - t.size(v): total += c.label

    // every other vertex
    var all = Fenwick<Size, Int>.new()    // size(v) over all v != root
    var seen = Fenwick<Size, Int>.new()   // size(v) over v visited so far
    var chain = Fenwick<Size, Int>.new()  // size(a) over proper ancestors a
    var above = Fenwick<Size, Int>.new()  // n - size(a) over ancestors a, self included
    for v in V:
        if v != c: all.add(Size(t.size(v)), 1)

    fn walk(x: V):
        let want = Size(max(n - 2 * t.size(x), 0))..=Size(n - 2 * heavy(x))
        above.add(Size(n - t.size(x)), 1)
        let before = seen.fold(want)
        seen.add(Size(t.size(x)), 1)
        let up = chain.fold(want)
        chain.add(Size(t.size(x)), 1)
        for k in t.children(x): walk(k)
        chain.add(Size(t.size(x)), -1)
        let below = seen.fold(want) - before
        total += x.label * (all.fold(want) - below - up + above.fold(want))
        above.add(Size(n - t.size(x)), -1)

    for k in t.children(c): walk(k)
    print(total)
