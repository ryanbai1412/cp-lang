// Runnable port (proto/cp.py lib-run) of statement-first ioi05p2.cp
// original: corpus/submissions/5032753.cpp
use std.algebra.{Monoid, Action}
use std.lazy.{LazySegTree}

let n: Int = read()
var tags: List<Int> = []
var ca: List<Int> = []
var cb: List<Int> = []
var cd: List<Int> = []

while true:
    let tok: Str = read()
    if tok[0] == 'E': break
    if tok[0] == 'I':
        let (a, b, d): (Int, Int, Int) = read()
        tags.push(0)
        ca.push(a)
        cb.push(b)
        cd.push(d)
    else:
        let h: Int = read()
        tags.push(1)
        ca.push(h)
        cb.push(0)
        cd.push(0)

struct SegVal:
    total: Int
    max_pre: Int
    len: Int

let rail_monoid = Monoid<SegVal>(
    id: SegVal(0, 0, 0),
    op: (x, y) => SegVal(x.total + y.total, max(x.max_pre, x.total + y.max_pre), x.len + y.len)
)

let assign_slope = Action<SegVal, ?Int>(
    id: none,
    compose: (f, g) => if f == none: g else: f,
    apply: (f, x, len) => if f == none: x else: SegVal(f! * x.len, max(0, f! * x.len), x.len),
    over: rail_monoid
)

var pts = [1, n + 1]
for i in 0..tags.len:
    if tags[i] == 0:
        pts.push(ca[i])
        pts.push(cb[i] + 1)
pts = sorted(pts)
var xs: List<Int> = []
for x in pts:
    if xs.len == 0 or xs[xs.len - 1] != x: xs.push(x)

fn coord(x: Int) -> Int:
    var (lo, hi) = (0, xs.len)
    while lo < hi:
        let mid = (lo + hi) / 2
        if xs[mid] < x: lo = mid + 1
        else: hi = mid
    lo

domain Seg = 0..(xs.len - 1)
let init = [Seg] i => SegVal(0, 0, xs[i.to_idx + 1] - xs[i.to_idx])
var st = LazySegTree<Seg, SegVal, ?Int>(assign_slope, init)

for i in 0..tags.len:
    if tags[i] == 0:
        let l = Seg(coord(ca[i]))
        let r = Seg(coord(cb[i] + 1) - 1)
        st.apply(l..=r, cd[i])
    else:
        let h = ca[i]
        // LazySegTree lacks max_right, so binary search over segment count today:
        if st.fold(Seg.first..=Seg.last).max_pre <= h:
            print(n)
        else:
            var (lo, hi) = (0, xs.len - 1)
            while lo < hi:
                let mid = (lo + hi) / 2
                if st.fold(Seg.first..=Seg(mid)).max_pre > h: hi = mid
                else: lo = mid + 1
            let seg_i = Seg(lo)
            let cur = if lo == 0: 0 else: st.fold(Seg.first..seg_i).total
            let node = st.get(seg_i)
            let slope = node.total / node.len
            let extra = (h - cur) / slope
            print(xs[lo] - 1 + extra)
