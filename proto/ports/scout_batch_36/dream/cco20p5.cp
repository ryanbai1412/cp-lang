// CCO '20 P5 — Interval Collection: maintain a multiset of closed intervals
// `[l, r]` (`1 <= l < r <= 1e6`) under `A l r` (add) and `R l r` (remove).
// After each operation, choose two (not necessarily distinct) intervals
// minimizing their intersection length, breaking ties by minimum bounding hull
// `max(r) - min(l)`, and print that hull.
//
// Offline segment tree over time (`TimeSegTree<Q, Span>`): pair each `Add` with
// its matching `Remove` (or `Q.last` if unclosed) so every interval lives on
// `O(log q)` tree nodes. During the DFS, maintain:
// - `min_len`: shortest active interval (handles single-interval multisets).
// - `min_r` (min `(r, -l)`) and `max_l` (min `(-l, r)`): when no disjoint pair
//   exists, the minimum overlap `r1 - l2` is achieved by `(min_r, max_l)` with
//   hull `max_l.r - min_r.l`.
// - `disjoint`: minimum hull `r2 - l1` over disjoint active pairs (`r1 <= l2`).
//   Two `RollbackFenwick<Coord, Max>` trees — forward `(l, r)` and mirrored
//   `(m - r, m - l)` — catch both arrival orders of a disjoint pair in `O(log M)`
//   and roll back automatically on DFS exit.

enum Kind: Add | Remove

struct Span(l: 1..=1_000_000, r: 1..=1_000_000)

struct Event(kind: Kind, span: Span)

struct State(disjoint: ?Int, min_r: ?Span, max_l: ?Span, min_len: ?Int)

let q: 1..=500_000 = read()
domain Q = 0..q

var events: List<Event> = []
var max_r = 1
for _ in Q:
    let tok: Str = read()
    let (l, r): (1..=1_000_000, 1..=1_000_000) = read()
    let kind = if tok[0] == 'A': Kind.Add else: Kind.Remove
    events.push(Event(kind, Span(l, r)))
    max_r max= r

let m = max_r + 2
domain Coord = 1..=m

var time_tree = TimeSegTree<Q, Span>()
var active = HashMap<(Int, Int), List<Q>>()
for i in Q:
    let ev = events[i.ord]
    let key = (ev.span.l, ev.span.r)
    match ev.kind:
        Kind.Add => active.entry(key).push(i)
        Kind.Remove =>
            let start = active[key].pop()!
            if let end = i.prev and start <= end: time_tree.add(start..=end, ev.span)

for (key, starts) in active:
    let sp = Span(key.0, key.1)
    for start in starts: time_tree.add(start..=Q.last, sp)

var fwd = RollbackFenwick<Coord, Max>()
var mir = RollbackFenwick<Coord, Max>()

fn insert_pair(mut bit: RollbackFenwick<Coord, Max>, l: Coord, r: Coord, cur: ?Int) -> ?Int:
    var best = cur
    if let hi = bit.prefix(l.r): best min= r.ord - hi
    bit.update(r, l.ord)
    best

fn pick_min(cur: ?Span, sp: Span, key: fn(Span) -> (Int, Int)) -> Span =
    if let b = cur:
        if key(sp) < key(b): sp else: b
    else:
        sp

var ans: [Q]?Int = none

fn enter(st: State, sp: Span) -> State:
    let d1 = insert_pair(mut fwd, Coord(sp.l), Coord(sp.r), st.disjoint)
    let d2 = insert_pair(mut mir, Coord(m - sp.r), Coord(m - sp.l), d1)
    var ml = st.min_len
    ml min= sp.r - sp.l
    State(
        d2,
        some(pick_min(st.min_r, sp, s => (s.r, -s.l))),
        some(pick_min(st.max_l, sp, s => (-s.l, s.r))),
        ml,
    )

fn answer_at(i: Q, st: State):
    var best = (st.min_len!, st.min_len!)
    let (left_end, right_start) = (st.min_r!, st.max_l!)
    if left_end.r >= right_start.l:
        best min= (left_end.r - right_start.l, right_start.r - left_end.l)
    if let d = st.disjoint:
        best min= (0, d)
    ans[i] = best.1

fn leave():
    fwd.rollback()
    mir.rollback()

time_tree.walk(State(none, none, none, none), enter, answer_at, leave)

for i in Q: print(ans[i]!)
