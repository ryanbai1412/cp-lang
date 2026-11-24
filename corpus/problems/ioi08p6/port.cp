// Source: IOI 2008 Day 2 (Pyramid Base), https://dmoj.ca/problem/ioi08p6
// Problem: an M x N grid (M, N <= 1e6) with P possibly overlapping rectangular obstacles, each
//   removable whole for cost C_i <= 7000. With budget B, print the largest side of a square base
//   whose cells can be cleared within B (0 if none). Either B = 0 and P <= 400000, or
//   0 < B <= 2e9 and P <= 30000.
// Solution: B > 0: binary search the side k. An obstacle charges its cost to a box of lowest
//   corners; sweep the corners' x with a lazy min/add SegTree over y and test the cheapest
//   corner: O((M + P) log N) per k. B = 0: two pointers over columns; a SegTree counting the
//   obstacles over each y (whole removals only, so no push-down) knows the longest free run, and
//   the window shrinks until a square of its width fits. O((M + P) log N).
// limits: 5000 ms, 256 MB
// original: corpus/submissions/5637805.cpp

let m: Int = read()
let n: Int = read()
let budget: Int = read()
let p: Int = read()
struct Box(x0: Int, y0: Int, x1: Int, y1: Int, cost: Int)
var boxes: List<Box> = []
for _ in 0..p:
    let x0: Int = read()
    let y0: Int = read()
    let x1: Int = read()
    let y1: Int = read()
    let cost: Int = read()
    boxes.push(Box(x0, y0, x1, y1, cost))

// the cheapest place for a base of side k, or none when the field is too small
fn cheapest(k: Int) -> Int:
    domain Y = 1..=n - k + 1
    // the cost changes only where a box starts or stops covering a corner x
    var events: List<(x: Int, by: Int, lo: Y, hi: Y)> = []
    for b in boxes:
        let (lo, hi) = (Y(max(b.y0 - k + 1, 1)), Y(min(b.y1, n - k + 1)))
        events.push((max(b.x0 - k + 1, 1), b.cost, lo, hi))
        if b.x1 < m - k + 1: events.push((b.x1 + 1, -b.cost, lo, hi))
    let order = sorted(events, by: e => e.x)
    var cost = SegTree<Y, Int, Int>(min, act: add, a: [Y] _ => 0)
    var best = if order.is_empty or order[0].x > 1: 0 else: Int.MAX
    var i = 0
    while i < order.len:
        let x = order[i].x
        while i < order.len and order[i].x == x:
            cost.apply(order[i].lo..=order[i].hi, order[i].by)
            i += 1
        best min= cost.fold(Y.all)
    best

fn with_budget() -> Int:
    (1..=min(m, n)).last_true(k => cheapest(k) <= budget) ?? 0  // side k fits

// How many intervals lie over each y in 1..=n, for intervals that are only
// ever removed whole: a node keeps the count of intervals that span it, and
// the free runs below it, so nothing is pushed down.
struct Cover(n: Int):
    over: List<Int>  // intervals spanning the whole node
    head: List<Int>  // free run at the node's start, end, and the longest
    tail: List<Int>
    run: List<Int>

    fn new(n: Int) -> Cover:
        var c = Cover(
            n,
            List.filled(4 * n, 0),
            List.filled(4 * n, 0),
            List.filled(4 * n, 0),
            List.filled(4 * n, 0),
        )
        c.build(1, 1, n)
        c

    fn build(mut self, at: Int, lo: Int, hi: Int):
        (head[at], tail[at], run[at]) = (hi - lo + 1, hi - lo + 1, hi - lo + 1)
        if lo == hi: return
        let mid = (lo + hi) / 2
        self.build(2 * at, lo, mid)
        self.build(2 * at + 1, mid + 1, hi)

    fn pull(mut self, at: Int, lo: Int, hi: Int):
        let (l, r, mid) = (2 * at, 2 * at + 1, (lo + hi) / 2)
        if over[at] > 0:
            (head[at], tail[at], run[at]) = (0, 0, 0)
        elif lo == hi:
            (head[at], tail[at], run[at]) = (1, 1, 1)
        else:
            head[at] = if head[l] == mid - lo + 1: head[l] + head[r] else: head[l]
            tail[at] = if tail[r] == hi - mid: tail[r] + tail[l] else: tail[r]
            run[at] = max(run[l], run[r], tail[l] + head[r])

    fn add(mut self, at: Int, lo: Int, hi: Int, from: Int, to: Int, by: Int):
        if from <= lo and hi <= to:
            over[at] += by
        else:
            let mid = (lo + hi) / 2
            if from <= mid: self.add(2 * at, lo, mid, from, to, by)
            if to > mid: self.add(2 * at + 1, mid + 1, hi, from, to, by)
        self.pull(at, lo, hi)

    fn put(mut self, from: Int, to: Int): self.add(1, 1, n, from, to, 1)
    fn take(mut self, from: Int, to: Int): self.add(1, 1, n, from, to, -1)
    @get fn longest_free(self) -> Int = run[1]

fn for_free() -> Int:
    domain X = 1..=m
    var enters: [X]List<Box> = []
    var leaves: [X]List<Box> = []
    for b in boxes:
        enters[X(b.x0)].push(b)
        leaves[X(b.x1)].push(b)
    var cover = Cover.new(n)
    var (best, left) = (0, 1)  // the window is columns left..=right
    for right in 1..=m:
        for b in enters[X(right)]: cover.put(b.y0, b.y1)
        while cover.longest_free < right - left + 1:
            for b in leaves[X(left)]: cover.take(b.y0, b.y1)
            left += 1
        best max= right - left + 1
    best

print(if budget > 0: with_budget() else: for_free())
