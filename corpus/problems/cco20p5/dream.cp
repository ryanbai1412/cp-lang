// CCO 2020 "Interval Collection": add and remove intervals; after each change print the shortest
// span over all sets whose intersection is smallest.
// Overlapping sets: the interval ending first and the one starting last give the smallest
// overlap, or a single interval does. Disjoint pairs (overlap 0) beat any overlap. All of it is
// kept offline over time: each copy is live on a slot range, and a rollback state tracks the
// first end, the last start, the shortest interval and the best disjoint span.
use std.timetree.{offline_over}

let q: 1..=500_000 = read()
domain Q = 0..q
let ops: [Q](tok: Str, l: Int, r: Int) = read()
let inf = 1_000_000_000
domain X = 0..=max(o.r for o in ops)! + 1

// In (l, r, time) order a remove pairs off with the latest unremoved add of its interval.
var live: List<(l: Int, r: Int, lo: Int, hi: Int)> = []
var open: List<Int> = []
for t in [t for t in Q].sorted(by: t => (ops[t].l, ops[t].r, t.ord)):
    let o = ops[t]
    if let i = open.last:
        if (live[i].l, live[i].r) != (o.l, o.r): open = []
    if o.tok == "A":
        open.push(live.len)
        live.push((l: o.l, r: o.r, lo: t.ord, hi: q))
    else: live[open.pop()!].hi = t.ord

// ending[x]: the latest start of a live interval ending at x; starting[x]: the earliest end of
// one starting at x. Every change is logged so a mark can roll it back.
var ending = SegTree<X, Int>(max, fill: -inf)
var starting = SegTree<X, Int>(min, fill: inf)
var log: List<(ends: Bool, at: X, was: Int)> = []
// end: least r, greatest l on ties, as (r, -l); start: greatest l, least r on ties, as (-l, r)
var s = (disjoint: inf, end: (inf, inf), start: (inf, inf), short: inf)
fn put(ends: Bool, at: X, x: Int):
    let was = if ends: ending.get(at) else: starting.get(at)
    log.push((ends:, at:, was:))
    if ends: ending.set(at, max(was, x)) else: starting.set(at, min(was, x))
fn add(iv: (l: Int, r: Int, lo: Int, hi: Int)):
    let before = ending.fold(X.first!..=X(iv.l))
    if before > -inf: s.disjoint = min(s.disjoint, iv.r - before)
    let after = starting.fold(X(iv.r)..=X.last!)
    if after < inf: s.disjoint = min(s.disjoint, after - iv.l)
    s.end = min(s.end, (iv.r, -iv.l))
    s.start = min(s.start, (-iv.l, iv.r))
    s.short = min(s.short, iv.r - iv.l)
    put(true, X(iv.r), iv.l)
    put(false, X(iv.l), iv.r)
fn span() -> Int:
    let ((ar, nal), (nbl, br)) = (s.end, s.start)
    var best = (overlap: s.short, span: s.short)
    if ar >= -nbl: best = min(best, (overlap: ar + nbl, span: br + nal))
    if s.disjoint < inf: best = min(best, (overlap: 0, span: s.disjoint))
    best.span
let spans = offline_over(q, live, by: iv => (iv.lo, iv.hi), mark: () => (
    len: log.len,
    s:,
), apply: add, answer: _ => span(), undo: m =>
    while log.len > m.len:
        let e = log.pop()!
        if e.ends: ending.set(e.at, e.was) else: starting.set(e.at, e.was)
    s = m.s,
)
for x in spans: print(x)
