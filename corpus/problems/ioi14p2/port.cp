// Source: IOI 2014 Day 1 (Wall), https://dmoj.ca/problem/ioi14p2
// limits: 1000 ms, 256 MB
// Problem: n <= 2e6 columns start at height 0. Apply k <= 5e5 operations `t l r h` (h <= 1e5)
//   in order: add (t = 1) raises every column of [l, r] below h to h, remove (t = 2) lowers
//   every column of [l, r] above h to h. Print the final heights.
// Solution: both operations clamp a height into an interval [lo, hi], and clamps compose
//   into a clamp. So it is a range update with no range query: a SegTree with a lazy clamp
//   action applies each operation in O(log n), then each column is read once:
//   O((n + k) log n).
// original: corpus/problems/ioi14p2/orig.cpp (submission 4683906 plus the sample grader)
// Bounds: n: 1..=2e6; k: 1..=5e5; op: 1..=2; l, r: 0..n; h: 0..=TALLEST

let (n, k): (1..=2e6, 1..=5e5) = read()
domain Col = 0..n
const TALLEST = 100_000
type Height = 0..=TALLEST

// clamp into [lo, hi]; the full range is the identity on every height
type Clamp = (lo: Height, hi: Height)
fn clamp(c: Clamp, h: Int) -> Int = min(max(h, c.lo), c.hi)
let clamps = Action<Int, Clamp>(
    (lo: 0, hi: TALLEST),
    (f, g) => (lo: clamp(f, g.lo), hi: clamp(f, g.hi)),
    (f, h, len) => clamp(f, h),
    over: max,
)

var wall = SegTree(max, [Col] c => 0, act: clamps)
for _ in 0..k:
    let (op, l, r, h): (1..=2, Col, Col, Height) = read()
    wall.apply(l..=r, if op == 1: (lo: h, hi: TALLEST) else: (lo: 0, hi: h))
for c in Col: print(wall.get(c))
