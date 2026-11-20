struct SegNode(mx: Int, pref: Int, suff: Int, all: Bool, cnt: Int):
    fn empty() -> SegNode = SegNode(0, 0, 0, false, 0)
    fn single(v: Int) -> SegNode = SegNode(v, 1, 1, true, 1)
    fn join(a: SegNode, b: SegNode) -> SegNode:
        let mx = max(a.mx, b.mx)
        let l = if a.mx == mx: a else: SegNode.empty()
        let r = if b.mx == mx: b else: SegNode.empty()
        SegNode(
            mx,
            l.pref + (if l.all: r.pref else: 0),
            r.suff + (if r.all: l.suff else: 0),
            l.all and r.all,
            l.cnt + r.cnt + l.suff * r.pref,
        )

@!multitest
let (n, q): (Int, Int) = read()
domain V = 0..n
var a: List<Int> = [read::<Int>() - 1 for _ in 0..n]
var locs: [0..n]Map<Int, Bool> = [0..n] _ => Map()
for i in 0..n: locs[a[i]][i] = true

var seg = SegTree<V, SegNode>(Monoid(SegNode.empty(), (x, y) => SegNode.join(x, y)))
for v in 0..n:
    if locs[v].len >= 2:
        let lo = locs[v].first_ge(0)!.0
        let hi = locs[v].last_le(n)!.0
        seg.set(V(lo), SegNode.single(hi - lo))

for _ in 0..q:
    let (x0, nv0): (Int, Int) = read()
    let x = x0 - 1
    let old_v = a[x]
    if locs[old_v].len >= 2:
        seg.set(V(locs[old_v].first_ge(0)!.0), SegNode.empty())
    locs[old_v].remove(x)
    if locs[old_v].len >= 2:
        let lo = locs[old_v].first_ge(0)!.0
        let hi = locs[old_v].last_le(n)!.0
        seg.set(V(lo), SegNode.single(hi - lo))
    let nv = nv0 - 1
    a[x] = nv
    if locs[nv].len >= 2:
        seg.set(V(locs[nv].first_ge(0)!.0), SegNode.empty())
    locs[nv][x] = true
    if locs[nv].len >= 2:
        let lo = locs[nv].first_ge(0)!.0
        let hi = locs[nv].last_le(n)!.0
        seg.set(V(lo), SegNode.single(hi - lo))
    let top = seg.fold(V(0)..)
    print(top.mx, top.cnt)
