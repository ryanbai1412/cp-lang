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
// Bounds: n: 1..=299995
// original: corpus/submissions/5148136.cpp

@!multitest
let n: 1..=299995 = read()
domain V = 1..=n
domain Size = 0..=n
let g = Graph.undirected(read<Tree<V>>())
let c = Rooted(g, root: V.first!).centroid()
let t = Rooted(g, root: c)
let heavy = [V] x => t.heavy[x].map(h => t.size(h)) ?? 0

var total = 0

// the root: its two largest children decide
let second = [t.size(k) for k in t.children(c) if Some(k) != t.heavy[c]].max() ?? 0
for v in V:
    if v == c: continue
    let rest = if t.is_anc(t.heavy[c]!, v): second else: heavy[c]
    if 2 * rest <= n - t.size(v): total += c.label

// every other vertex
var all = Fenwick<Size, Int>.new()    // size(v) over all v != root
var seen = Fenwick<Size, Int>.new()   // size(v) over v visited so far
var chain = Fenwick<Size, Int>.new()  // size(a) over proper ancestors a
var above = Fenwick<Size, Int>.new()  // n - size(a) over ancestors a, self included
for v in V:
    if v != c: all.add(Size(t.size(v)), 1)

fn walk(x: V):
    let want = Size(max(n - 2 * t.size(x), 0))..=Size(n - 2 * heavy[x])
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
