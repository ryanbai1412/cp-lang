// IOI 2014 Wall: n columns of height 0 and k operations in order. "add"
// raises every column of a range that is below h up to h; "remove" lowers
// every column above h down to h. Print the final heights.
//
// Both operations clamp a height to an interval, and clamps compose into a
// clamp. So this is a range update with no range query: push the clamps
// down a lazy tree and read each leaf at the end.

let (n, k): (Int, Int) = read()
domain Col = 0..n

type Clamp = (lo: Int, hi: Int)
fn clamp(c: Clamp, h: Int) -> Int = min(max(h, c.lo), c.hi)
let clamps = Action<Int, Clamp>(
    (lo: 0, hi: Int.MAX),
    (f, g) => (lo: clamp(f, g.lo), hi: clamp(f, g.hi)),
    (f, h, len) => clamp(f, h),
    over: max,
)

var wall = LazySegTree<Col, Int, Clamp>(clamps, [Col] c => 0)
for _ in 0..k:
    let (op, l, r, h): (Int, Col, Col, Int) = read()
    wall.apply(l..=r, if op == 1: (lo: h, hi: Int.MAX) else: (lo: 0, hi: h))
for c in Col: print(wall.get(c))
