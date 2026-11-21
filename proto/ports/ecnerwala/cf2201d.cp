// Codeforces 2201 D: maintain each value's span [first, last] in a segment tree
// tracking maximum span and number of subarrays achieving it.

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
let (n, q): (Int, Int) = read()
domain V = 1..=n
var a: [V]V = read()
var locs: [V]Set<V> = [V] _ => Set()
for i in V: locs[a[i]].add(i)

// The cell at a value's first position carries its span (last - first).
var seg = SegTree<V, SegNode>(Monoid(SegNode.empty(), (x, y) => SegNode.join(x, y)))

fn sync(v: V):
    if let lo = locs[v].first, let hi = locs[v].last, lo != hi:
        seg.set(lo, SegNode.single(hi.ord - lo.ord))

for v in V: sync(v)

repeat q:
    let (x, nv): (V, V) = read()
    let old = a[x]
    for v in [old, nv]:
        if let lo = locs[v].first, locs[v].len >= 2: seg.set(lo, SegNode.empty())
    locs[old].remove(x)
    a[x] = nv
    locs[nv].add(x)
    for v in [old, nv]: sync(v)

    let top = seg.fold(V.first..)
    print(top.mx, top.cnt)
