// IOI '05 P2 - Mountain (DMOJ: ioi05p2, 20pt, TL 1.4s, ML 256MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// A rollercoaster of n <= 1e9 unit rails starts flat (slope 0 everywhere).
// Commands until 'E':
//   I a b D : set rails a..=b to slope D (|D| <= 1e9)
//   Q h     : largest x in 0..=n such that height at every point 0..=x is <= h
//   E       : end of input
//
// Statement-first choice: read commands offline and coordinate-compress all
// endpoints (1, a, b + 1, n + 1) into elementary constant-slope segments.
// A LazySegTree over segments maintains (total_delta, max_prefix, len) with
// range-assign slope action. Answering Q h walks the tree (max_right) to the
// first segment exceeding h, then divides (h - cur) / slope inside that segment.
// This avoids dynamic node pools and stays well inside 256 MB.
use std.algebra.{Monoid, Action}
use std.lazy.{LazySegTree}

let n: 1..=1e9 = read()

// WISH W7: sentinel-terminated heterogeneous command stream (`while tag != 'E'`)
// cannot be expressed as a typed `read()` for `cpl stress` without a custom `gen` block.
enum Cmd:
    @tag("I") Set(a: 1..=n, b: 1..=n, d: -1e9..=1e9)
    @tag("Q") Ask(h: 0..=1e9)
    @tag("E") End

var cmds: List<Cmd> = []
while true:
    let c: Cmd = read()
    match c:
        End => break
        _ => cmds.push(c)

struct SegVal:
    total: Int
    max_pre: Int
    len: Int

const RailMonoid = Monoid<SegVal>(
    id: SegVal(0, 0, 0),
    op: (x, y) => SegVal(x.total + y.total, max(x.max_pre, x.total + y.max_pre), x.len + y.len)
)

const AssignSlope = Action<SegVal, ?Int>(
    id: none,
    compose: (f, g) => if f == none: g else: f,
    apply: (f, x) => match f:
        none => x
        d => SegVal(d * x.len, max(0, d * x.len), x.len)
)

fn solve() -> List<Int>:
    var pts = [1, n + 1]
    for c in cmds:
        match c:
            Set(a, b, _) =>
                pts.push(a)
                pts.push(b + 1)
            _ => ()
    pts.sort()
    // WISH W8: `pts.dedup()` in stdlib for coordinate compression
    var xs: List<Int> = []
    for x in pts:
        if xs.is_empty or xs.last != x: xs.push(x)

    domain Seg = 0..(xs.len - 1)
    let init = [Seg] i => SegVal(0, 0, xs[i.to_idx + 1] - xs[i.to_idx])
    var st = LazySegTree<Seg, SegVal, ?Int>.from(RailMonoid, AssignSlope, init)

    fn coord(x: Int) -> Int:
        var (lo, hi) = (0, xs.len)
        while lo < hi:
            let mid = (lo + hi) / 2
            if xs[mid] < x: lo = mid + 1
            else: hi = mid
        lo

    var out: List<Int> = []
    for c in cmds:
        match c:
            Set(a, b, d) =>
                let l = Seg(coord(a))
                let r = Seg(coord(b + 1) - 1)
                st.apply(l..=r, d)
            Ask(h) =>
                // WISH W9: `LazySegTree.max_right` is missing from `stdlib/lazy.cp`
                // (only `SegTree` has `max_right`, forcing a manual tree walk or O(log^2 n) bsearch).
                let stop = st.max_right(Seg.first.l, acc => acc.max_pre <= h)
                if stop == Seg.last.r:
                    out.push(n)
                else:
                    let seg_i = stop.right!
                    let cur = st.fold(Seg.first.l..stop).total
                    let node = st.get(seg_i)
                    let slope = node.total / node.len
                    let extra = (h - cur) / slope
                    out.push(xs[seg_i.to_idx] - 1 + extra)
            End => ()
    out

@brute(solve)
fn solve_brute() -> List<Int>
    where n <= 500 and cmds.len <= 200:
    var slope = List.filled(n + 1, 0)
    var out: List<Int> = []
    for c in cmds:
        match c:
            Set(a, b, d) =>
                for i in a..=b: slope[i] = d
            Ask(h) =>
                var (cur, ans) = (0, 0)
                for i in 1..=n:
                    cur += slope[i]
                    if cur > h: break
                    ans = i
                out.push(ans)
            End => ()
    out

for ans in solve(): print(ans)
