// Source: IOI 2008 Day 2 (Pyramid Base), https://dmoj.ca/problem/ioi08p6
// Problem: an M x N grid (M, N <= 1e6) with P possibly overlapping rectangular obstacles, each
//   removable whole for cost C_i <= 7000. With budget B, print the largest side of a square base
//   whose cells can be cleared within B (0 if none). Either B = 0 and P <= 400000, or
//   0 < B <= 2e9 and P <= 30000.
// Solution: B > 0: binary search the side k. An obstacle charges its cost to a box of lowest
//   corners; sweep the corners' x with a lazy min/add SegTree over y and test the cheapest
//   corner: O((M + P) log N) per k. B = 0: two pointers over columns; a range-add SegTree over y
//   keeps each node's least cover count and its runs at that count, so the longest run at count
//   0 is the widest free strip, and the window shrinks until a square of its width fits.
//   O((M + P) log N).
// limits: 5000 ms, 256 MB
// original: corpus/submissions/5637805.cpp
// Bounds: m, n: 1..=1e6; budget: 0..=2e9; p: 1..=4e5; boxes.x0, boxes.x1: 1..=m; boxes.y0, boxes.y1: 1..=n; boxes.cost: 1..=7000
let (m, n): (1..=1e6, 1..=1e6) = read()
let budget: 0..=2e9 = read()
let p: 1..=4e5 = read()
let boxes: [p](x0: 1..=m, y0: 1..=n, x1: 1..=m, y1: 1..=n, cost: 1..=7000) = read()

// The cheapest place for a base of side k <= min(m, n).
fn cheapest(k: Int) -> Int:
    domain X = 1..=m - k + 1  // where the base's lowest corner can be
    domain Y = 1..=n - k + 1
    var starts: [X]List<(lo: Y, hi: Y, cost: Int)> = []
    var stops: [X]List<(lo: Y, hi: Y, cost: Int)> = []
    for b in boxes:
        let span = (lo: Y(max(b.y0 - k + 1, 1)), hi: Y(min(b.y1, n - k + 1)), cost: b.cost)
        starts[X(max(b.x0 - k + 1, 1))].push(span)
        if let after = X.at(b.x1 + 1): stops[after].push(span)
    var cost = SegTree<Y, Int, Int>(min, act: add, a: [Y] _ => 0)
    var best: ?Int = None
    for x in X:
        for s in stops[x]: cost.apply(s.lo..=s.hi, -s.cost)
        for s in starts[x]: cost.apply(s.lo..=s.hi, s.cost)
        best min= cost.all
    best!

fn with_budget() -> Int:
    (1..=min(m, n)).last_true(k => cheapest(k) <= budget) ?? 0  // side k fits

// A run of rows: its least cover count, and the runs of rows at that count at its start, at its
// end and anywhere in it.
type Strip = (low: Int, len: Int, head: Int, tail: Int, run: Int)

fn at_low(s: Strip, low: Int) -> Strip = if s.low == low: s else: (low: s.low, len: s.len, head: 0, tail: 0, run: 0)

fn join(a: Strip, b: Strip) -> Strip:
    let low = min(a.low, b.low)
    let (l, r) = (at_low(a, low), at_low(b, low))
    (
        low:,
        len: a.len + b.len,
        head: if l.head == a.len: a.len + r.head else: l.head,
        tail: if r.tail == b.len: b.len + l.tail else: r.tail,
        run: max(l.run, r.run, l.tail + r.head),
    )

fn for_free() -> Int:
    domain X = 1..=m
    domain Y = 1..=n
    var enters: [X]List<Range<Y>> = []
    var leaves: [X]List<Range<Y>> = []
    for b in boxes:
        enters[X(b.x0)].push(Y(b.y0)..=Y(b.y1))
        leaves[X(b.x1)].push(Y(b.y0)..=Y(b.y1))
    let none: Strip = (low: Int.MAX, len: 0, head: 0, tail: 0, run: 0)
    let cover_by = Action<Strip, Int>(apply: (d, s, _) => (low: s.low + d, len: s.len, head: s.head, tail: s.tail, run: s.run), over: Monoid(none, join))
    var cover = SegTree<Y, Strip, Int>(Monoid(none, join), act: cover_by, a: [Y] _ => (low: 0, len: 1, head: 1, tail: 1, run: 1))
    fn widest_free() -> Int = if cover.all.low == 0: cover.all.run else: 0
    var (best, left) = (0, X.cuts.first!)  // the window is the columns from cut left to right
    for right in X:
        for r in enters[right]: cover.apply(r, 1)
        while widest_free() < Int(right.r) - Int(left):
            for r in leaves[left.r!]: cover.apply(r, -1)
            left = left.next!
        best max= Int(right.r) - Int(left)
    best

print(if budget > 0: with_budget() else: for_free())
