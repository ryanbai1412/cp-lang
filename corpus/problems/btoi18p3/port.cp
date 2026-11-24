// Source: Baltic OI 2018 Day 1 P3 (Worm Worries), https://dmoj.ca/problem/btoi18p3
// Problem: interactive. An N x M x K box (each <= 1e6) of hidden humidities; "? x y z" asks one,
//   at most Q times; answer "! x y z" with a cell no lower than its up to six neighbours. Tests
//   are a line (N = 1e6, Q = 35), a sheet (N = M <= 1000, Q = 3500) or a box (<= 500^3).
// Solution: a line: golden-section search, which reuses one probe each round. A sheet: halve the
//   longer side along a line and take its highest cell c; if the best cell seen so far is
//   higher, recurse into its half; else c is the answer if both cells across the line are lower,
//   else recurse towards the higher one. Each step reads one line: O(N + M) queries. A box: best
//   of Q / 2 random cells, then walk uphill. Heights are memoised; a cell off the box reads 0.
// dream.cp has the form with a product-domain sheet and `@memo` questions.
// original: corpus/submissions/4656860.cpp
use std.rand.{Rng}

let (n, m, k, q): (1..=1e6, 1..=1e6, 1..=1e6, 1..=1e5) = read()
type Cell = (Int, Int, Int)  // 0-based; printed 1-based

var known = Map<Cell, Int>()
var asked = 0
fn height(c: Cell) -> Int:
    let (x, y, z) = c
    if not (0 <= x < n and 0 <= y < m and 0 <= z < k): return 0
    if let h = known.get(c): return h
    asked += 1
    assert asked <= q
    let h: Int = ask("? {x + 1} {y + 1} {z + 1}")
    known[c] = h
    h

const STEPS = [(0, 0, -1), (0, 0, 1), (0, -1, 0), (0, 1, 0), (-1, 0, 0), (1, 0, 0)]
fn beside(c: Cell) -> List<Cell> = [(c.0 + d.0, c.1 + d.1, c.2 + d.2) for d in STEPS]

// A line: golden-section search; each round keeps one probe for the next.
fn on_line() -> Int:
    let ra = (sqrt(5.0) - 1.0) / 2.0
    fn cuts(l: Int, r: Int) -> (Int, Int) =
        (trunc(F64(l) * ra + F64(r) * (1.0 - ra)), trunc(F64(l) * (1.0 - ra) + F64(r) * ra))
    fn at(x: Int) -> Int = height((x, 0, 0))
    var (l, r) = (0, n - 1)
    var (m1, m2) = cuts(l, r)
    while m1 < m2:
        if at(m1) < at(m2):
            l = m1
            let kept = m2
            (m1, m2) = cuts(l, r)
            m1 = kept
        else:
            r = m2
            let kept = m1
            (m1, m2) = cuts(l, r)
            m2 = kept
    (l..r).find(x => at(x) >= max(at(x - 1), at(x + 1))) ?? r

// A sheet [x1, x2] x [y1, y2]: cut the longer side along a line through its middle. (px, py)
// with height pv is the best cell seen on an earlier line, or pv = -1.
fn on_sheet(x1: Int, y1: Int, x2: Int, y2: Int, px: Int, py: Int, pv: Int) -> ?(Int, Int):
    if x1 > x2 or y1 > y2: return None
    fn at(x: Int, y: Int) -> Int = height((x, y, 0))
    if x2 - x1 <= y2 - y1:
        let mid = (y1 + y2) / 2
        let (best, x) = [(at(i, mid), i) for i in x1..=x2].max()!
        if pv >= best:
            let half = if py <= mid: on_sheet(x1, y1, x2, mid - 1, px, py, pv) else: on_sheet(x1, mid + 1, x2, y2, px, py, pv)
            if let found = half: return Some(found)
        let a = if mid - 1 >= y1: at(x, mid - 1) else: 0
        let c = if mid + 1 <= y2: at(x, mid + 1) else: 0
        if best >= a and best >= c: return Some((x, mid))
        if a >= c: on_sheet(x1, y1, x2, mid - 1, x, mid, best) else: on_sheet(x1, mid + 1, x2, y2, x, mid, best)
    else:
        let mid = (x1 + x2) / 2
        let (best, y) = [(at(mid, i), i) for i in y1..=y2].max()!
        if pv >= best:
            let half = if px <= mid: on_sheet(x1, y1, mid - 1, y2, px, py, pv) else: on_sheet(mid + 1, y1, x2, y2, px, py, pv)
            if let found = half: return Some(found)
        let a = if mid - 1 >= x1: at(mid - 1, y) else: 0
        let c = if mid + 1 <= x2: at(mid + 1, y) else: 0
        if best >= a and best >= c: return Some((mid, y))
        if a >= c: on_sheet(x1, y1, mid - 1, y2, mid, y, best) else: on_sheet(mid + 1, y1, x2, y2, mid, y, best)

// A box: the best of q / 2 random cells, then walk uphill.
fn in_box() -> Cell:
    var rng = Rng(1)
    var c = [(rng.int(0, n - 1), rng.int(0, m - 1), rng.int(0, k - 1)) for _ in 0..q / 2].max(by: height)!
    while let d = beside(c).find(d => height(d) > height(c)): c = d
    c

let found: Cell =
    if m == 1 and k == 1: (on_line(), 0, 0)
    elif q >= 100_000: in_box()
    else:
        let (x, y) = on_sheet(0, 0, n - 1, m - 1, -1, -1, -1)!
        (x, y, 0)
print("! {found.0 + 1} {found.1 + 1} {found.2 + 1}")
