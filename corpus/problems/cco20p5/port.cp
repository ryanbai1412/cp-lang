// Source: CCO 2020 P5 (Interval Collection), https://dmoj.ca/problem/cco20p5
// limits: 3500 ms, 1024 MB
// Problem: Q <= 5e5 events add or remove one copy of an interval [l, r] (1 <= l < r <= 1e6). After
//   each, pick a non-empty subset S: prefer one whose intervals share no common interval of
//   positive length, else the smallest such greatest common interval, then the shortest least
//   enclosing interval; print that enclosing length.
// Solution: online. Without a disjoint pair, the best set is the interval with the least r
//   (greatest l on ties) with the one with the greatest l (least r on ties), if they overlap,
//   else the shortest interval alone; std `SortedBag`s of (r, l), (l, r) and lengths give
//   those. A disjoint pair (r' <= l) beats any overlap; a std `SegTree` over positions keeps,
//   per range, the greatest l of an interval ending there, the least r of one starting there
//   and the shortest span r - l' of a pair ending before (or at) where the other starts.
//   O(Q log C).
// original: corpus/submissions/4562195.cpp
// Bounds: q: 1..=500_000; events.l, events.r: 1..=1_000_000
let q: 1..=500_000 = read()
domain Event = 0..q
domain X = 1..=1_000_000
let events: [Event](op: Char, l: X, r: X) = read()

fn least(a: ?Int, b: ?Int) -> ?Int = if let x = a: Some(min(x, b ?? x)) else: b

// intervals ending in a range (greatest l), starting in it (least r), and the shortest span
// of a pair whose first ends no later than the second starts
struct Gaps(end_l: ?Int, start_r: ?Int, span: ?Int):
    const none = Gaps(None, None, None)
    fn join(a: Gaps, b: Gaps) -> Gaps:
        let across = if let (Some(l), Some(r)) = (a.end_l, b.start_r): Some(r - l) else: None
        Gaps(
            max(a.end_l, b.end_l),
            least(a.start_r, b.start_r),
            least(least(a.span, b.span), across),
        )

// ends: least r first, greatest l on ties; starts: greatest l first, least r on ties
var ends = SortedBag<(r: Int, l: Desc<Int>)>()
var starts = SortedBag<(l: Desc<Int>, r: Int)>()
var lens = SortedBag<Int>()
var gaps = SegTree<X, Gaps>(Monoid(Gaps.none, Gaps.join))

fn leaf(end_l: ?Int, start_r: ?Int) -> Gaps = Gaps.join(
    Gaps(end_l, None, None),
    Gaps(None, start_r, None),
)

for e in events:
    let (l, r) = (Int(e.l), Int(e.r))
    if e.op == 'A':
        ends.add((r:, l: desc(l)))
        starts.add((l: desc(l), r:))
        lens.add(r - l)
    else:
        ends.remove((r:, l: desc(l)))
        starts.remove((l: desc(l), r:))
        lens.remove(r - l)
    // only the greatest l ending at r and the least r starting at l moved
    let end_l = ends.first_true(x => x.r >= r).filter(x => x.r == r).map(x => x.l.x)
    let start_r = starts.first_true(x => x.l.x <= l).filter(x => x.l.x == l).map(x => x.r)
    gaps.set(e.r, leaf(end_l, gaps.get(e.r).start_r))
    gaps.set(e.l, leaf(gaps.get(e.l).end_l, start_r))
    let (a, b) = (ends.first!, starts.first!)
    var best = (overlap: lens.first!, span: lens.first!)
    if a.r >= b.l.x: best min= (overlap: a.r - b.l.x, span: b.r - a.l.x)
    if let span = gaps.all.span: best min= (overlap: 0, span:)
    print(best.span)
