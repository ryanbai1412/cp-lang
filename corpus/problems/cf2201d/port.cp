// Source: Codeforces 2201D (Binary Not Search and Queries),
//   https://codeforces.com/contest/2201/problem/D
// limits: 4000 ms, 512 MB
// Problem: an array a of n <= 2e5 values in 1..n gets q <= 1e5 point assignments. After each,
//   print k_max, the largest k such that two different length-k windows i < j hold the same
//   multiset, and f, the number of such (i, j, k) with k = k_max (0 0 if none). Sums of n
//   and q over a multitest are bounded the same way.
// Solution: k_max is the largest span last(v) - first(v) of a value (windows i and i + 1
//   differ only in a[i] and a[i + k]), and the optimal tuples are the runs of consecutive
//   positions that are each the first occurrence of a value with that span. A SegTree over
//   positions holds each value's span at its first occurrence; a node keeps the max span,
//   its prefix and suffix runs, whether all leaves reach it and the count of such runs. Each
//   value's positions sit in an ordered Set; an update resyncs the old and the new value:
//   O(log n) per query.

struct SegNode(mx: Int, pref: Int, suff: Int, all: Bool, cnt: Int):
    fn empty() -> SegNode = SegNode(0, 0, 0, false, 0)
    fn single(v: Int) -> SegNode = SegNode(v, 1, 1, true, 1)
    fn join(x: SegNode, y: SegNode) -> SegNode:
        let mx = max(x.mx, y.mx)
        let l = if x.mx == mx: x else: SegNode.empty()
        let r = if y.mx == mx: y else: SegNode.empty()
        SegNode(
            mx,
            l.pref + (if l.all: r.pref else: 0),
            r.suff + (if r.all: l.suff else: 0),
            l.all and r.all,
            l.cnt + r.cnt + l.suff * r.pref,
        )

@!multitest
let (n, q): (1..=200_000, 1..=200_000) = read()
domain V = 1..=n
var a: [V]V = read()
var locs: [V]Set<V> = [V] _ => Set()
for i in V: locs[a[i]].add(i)

// The cell at a value's first position carries its span (last - first).
var seg = SegTree<V, SegNode>(Monoid(SegNode.empty(), (x, y) => SegNode.join(x, y)))

fn span_of(v: V) -> ?(lo: V, span: Int):
    if let lo = locs[v].first, let hi = locs[v].last, lo != hi:
        Some((lo, hi.ord - lo.ord))
    else:
        None

fn clear(v: V):
    if let s = span_of(v): seg.set(s.lo, SegNode.empty())

fn sync(v: V):
    if let s = span_of(v): seg.set(s.lo, SegNode.single(s.span))

for v in V: sync(v)

repeat q:
    let (x, nv): (V, V) = read()
    let old = a[x]
    for v in [old, nv]: clear(v)
    locs[old].remove(x)
    a[x] = nv
    locs[nv].add(x)
    for v in [old, nv]: sync(v)

    let top = seg.all
    print(top.mx, top.cnt)
