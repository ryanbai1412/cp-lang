// Codeforces 2201 D: maintain each value's span `[first, last]` at its first
// position `first` in a segment tree, tracking the maximum span `mx` and the
// number of contiguous subarrays of positions whose spans all equal `mx`.
//
// `SegNode(mx, pref, suff, all, cnt)` invariant over a segment of leaves:
// - `mx`: maximum span in the segment (0 if no value repeats here).
// - `pref` / `suff`: length of the longest prefix / suffix of leaves with span `== mx`.
// - `all`: true iff every leaf in the segment has span `== mx`.
// - `cnt`: number of non-empty subarrays in the segment whose leaves all equal `mx`.
// On `join(x, y)`, any child with max `< max(x.mx, y.mx)` contributes no
// `mx`-leaves and is replaced by `empty()`. A valid subarray then lies wholly
// in `l` (`l.cnt`), wholly in `r` (`r.cnt`), or crosses the midpoint by pairing
// one of the `l.suff` trailing `mx`-leaves with one of the `r.pref` leading
// `mx`-leaves (`l.suff * r.pref`).

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
        some((lo, hi.ord - lo.ord))
    else:
        none

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

    let top = seg.fold(V.first..)
    print(top.mx, top.cnt)
