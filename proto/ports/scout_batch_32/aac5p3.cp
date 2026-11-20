// AAC 5 P3 (DMOJ: aac5p3) - values routed down a weighted tree
// Ideal v2 port. original: corpus/submissions/4239862.cpp
//
// m values enter at the root. At a vertex each value moves along the child
// edge whose weight is closest to it (ties go to the smaller weight), so the
// children, sorted by weight, cut the incoming interval of values at the
// midpoints between neighbouring weights. Print how many values visit each vertex.
let (n, m): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
let es: [0..n - 1](u: V, v: V, w: 1..=1e9) = read()
let vals: [0..m]Int = read()
// WISH X8: the cheat sheet only shows empty constructors (`SortedBag<T>()`);
// building an ordered container from a sequence is assumed here.
let xs = SortedBag<Int>(vals)
let g = Graph.undirected(es)

var ans: [V]Int = 0
fn route(u: V, from: ?V, part: Range<Int>):
    if part.is_empty: return
    ans[u] = xs.rank(part.hi) - xs.rank(part.lo)
    let kids = sorted((w: w, v: v) for (v, w) in g[u] if v != from)
    // cuts[i]..cuts[i + 1] is what child i receives.
    let cuts = [part.lo] + [(a.w + b.w) / 2 + 1 for (a, b) in kids.adjacent] + [part.hi]
    for (i, kid) in kids.items:
        // WISH X5: intersecting two half-open ranges is spelled with max/min by hand.
        route(kid.v, u, max(cuts[i], part.lo)..min(cuts[i + 1], part.hi))

route(V.first, none, 0..1e9 + 5)
print(" ".join("{c}" for c in ans))
