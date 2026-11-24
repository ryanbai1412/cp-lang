// Source: IOI 2014 Day 1 (Wall), https://dmoj.ca/problem/ioi14p2
// Problem: n <= 2e6 columns start at height 0. Apply k <= 5e5 operations `t l r h` (h <= 1e5)
//   in order: add (t = 1) raises every column of [l, r] below h to h, remove (t = 2) lowers
//   every column of [l, r] above h to h. Print the final heights.
// Solution: both operations clamp a height into an interval [lo, hi], and clamps compose
//   into a clamp. So it is a range update with no range query: a SegTree with a lazy clamp
//   action applies each operation in O(log n), then each column is read once:
//   O((n + k) log n).

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

var wall = SegTree(max, [Col] c => 0, act: clamps)
for _ in 0..k:
    let (op, l, r, h): (Int, Col, Col, Int) = read()
    wall.apply(l..=r, if op == 1: (lo: h, hi: Int.MAX) else: (lo: 0, hi: h))
for c in Col: print(wall.get(c))
