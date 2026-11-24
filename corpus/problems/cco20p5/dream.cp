// CCO 2020 "Interval Collection": add and remove intervals; after each change print the shortest
// span over all sets whose intersection is smallest.
// Overlapping sets: the interval ending first and the one starting last give the smallest
// overlap, or a single interval does. Disjoint pairs (overlap 0) are kept offline over time:
// each interval is live on a slot range, and a rollback state tracks the best disjoint span.
use std.timetree.{offline_over}

let q: 1..=500_000 = read()
domain Q = 0..q
let ops: [Q](tok: Str, l: Int, r: Int) = read()
let inf = 1_000_000_000
domain X = 0..=max(o.r for o in ops)! + 1

// The slots each added interval is live on.
var open = HashMap<(Int, Int), List<Int>>()
var live: List<(l: Int, r: Int, lo: Int, hi: Int)> = []
for t in Q:
    let o = ops[t]
    if open.get((o.l, o.r)) == None: open[(o.l, o.r)] = []
    if o.tok == "A": open[(o.l, o.r)].push(t.ord)
    else: live.push((l: o.l, r: o.r, lo: open[(o.l, o.r)].pop()!, hi: t.ord))
for ((l, r), starts) in open.items:
    for s in starts: live.push((l:, r:, lo: s, hi: q))

// ending[x]: the latest start of a live interval ending at x; starting[x]: the earliest end of
// one starting at x. Every change is logged so a mark can roll it back.
var ending = SegTree<X, Int>(max, fill: -inf)
var starting = SegTree<X, Int>(min, fill: inf)
var log: List<(ends: Bool, at: X, was: Int)> = []
var disjoint = inf
fn put(ends: Bool, at: X, x: Int):
    let was = if ends: ending.get(at) else: starting.get(at)
    log.push((ends:, at:, was:))
    if ends: ending.set(at, max(was, x)) else: starting.set(at, min(was, x))
fn add(iv: (l: Int, r: Int, lo: Int, hi: Int)):
    let before = ending.fold(X.first!..=X(iv.l))
    if before > -inf: disjoint = min(disjoint, iv.r - before)
    let after = starting.fold(X(iv.r)..=X.last!)
    if after < inf: disjoint = min(disjoint, after - iv.l)
    put(true, X(iv.r), iv.l)
    put(false, X(iv.l), iv.r)
let best_disjoint = offline_over(q, live, by: iv => (iv.lo, iv.hi),
    mark: () => (len: log.len, best: disjoint),
    apply: add,
    answer: _ => disjoint,
    undo: m =>
        while log.len > m.len:
            let e = log.pop()!
            if e.ends: ending.set(e.at, e.was) else: starting.set(e.at, e.was)
        disjoint = m.best,
)

var by_end = SortedBag<(r: Int, l: Desc<Int>)>()
var by_start = SortedBag<(l: Desc<Int>, r: Int)>()
var lens = SortedBag<Int>()
for t in Q:
    let o = ops[t]
    if o.tok == "A":
        by_end.add((r: o.r, l: desc(o.l)))
        by_start.add((l: desc(o.l), r: o.r))
        lens.add(o.r - o.l)
    else:
        by_end.remove((r: o.r, l: desc(o.l)))
        by_start.remove((l: desc(o.l), r: o.r))
        lens.remove(o.r - o.l)
    var best = (overlap: lens.first!, span: lens.first!)
    let (a, b) = (by_end.first!, by_start.first!)
    if a.r >= b.l.x:
        best = min(best, (overlap: a.r - b.l.x, span: b.r - a.l.x))
    if best_disjoint[t.ord] < inf:
        best = min(best, (overlap: 0, span: best_disjoint[t.ord]))
    print(best.span)
