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

@! multitest
let (n, q): (Int, Int) = read()
domain V = 1..=n
var a: [V]V = read()
var locs: [V]Set<Int> = [V] _ => Set()
for i in V: locs[a[i]].add(i.to_idx)

var seg = SegTree<V, SegNode>(Monoid(SegNode.empty(), (x, y) => SegNode.join(x, y)))

fn clear_val(v: V):
    if locs[v].len >= 2: seg.set(V.nth(locs[v].first!), SegNode.empty())

fn sync_val(v: V):
    if locs[v].len >= 2:
        let (lo, hi) = (locs[v].first!, locs[v].last!)
        seg.set(V.nth(lo), SegNode.single(hi - lo))

for v in V: sync_val(v)

for _ in 0..q:
    let (x, nv): (V, V) = read()
    let old_v = a[x]
    clear_val(old_v)
    locs[old_v].remove(x.to_idx)
    sync_val(old_v)

    a[x] = nv
    clear_val(nv)
    locs[nv].add(x.to_idx)
    sync_val(nv)

    let top = seg.fold(V(1)..)
    print(top.mx, top.cnt)
