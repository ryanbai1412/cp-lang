// CCO '20 P5 — Interval Collection: maintain a multiset of closed intervals
// `[l, r]` (`1 <= l < r <= 1e6`) under `A l r` (add) and `R l r` (remove).
// After each operation, choose two (not necessarily distinct) intervals
// minimizing their intersection length, breaking ties by minimum bounding hull
// `max(r) - min(l)`, and print that hull.
//
// Offline segment tree over time (`0..q`): pair each `Add` with its matching
// `Remove` (or `q - 1` if unclosed) so every interval lives on `O(log q)` tree
// nodes. During the DFS, maintain:
// - `min_len`: shortest active interval (handles single-interval multisets).
// - `min_r` (min `r`, tie max `l`) and `max_l` (max `l`, tie min `r`): when no
//   disjoint pair exists, the minimum overlap `r1 - l2` is achieved by `(min_r, max_l)`
//   with hull `max_l.r - min_r.l`.
// - `disjoint`: minimum hull `r2 - l1` over disjoint active pairs (`r1 <= l2`).
//   Two rollback prefix-max Fenwicks — forward `(l, r)` and mirrored
//   `(M - r, M - l)` — catch both arrival orders of a disjoint pair in `O(log M)`
//   with an explicit undo log on DFS exit.

enum Kind: Add | Remove

struct Span(l: 1..=1_000_000, r: 1..=1_000_000)

struct Event(kind: Kind, span: Span)

struct State(disjoint: ?Int, min_r: ?Span, max_l: ?Span, min_len: ?Int)

struct Undo(side: 0..2, pos: Int, prev: Int)

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
var seg: [0..4 * q + 5]List<Span> = []

fn add_span(o: Int, l: Int, r: Int, ql: Int, qr: Int, sp: Span):
    if ql <= l and r <= qr:
        seg[o].push(sp)
        return
    let mid = (l + r) / 2
    if ql <= mid: add_span(o * 2, l, mid, ql, qr, sp)
    if mid < qr: add_span(o * 2 + 1, mid + 1, r, ql, qr, sp)

var active = HashMap<(Int, Int), List<Int>>()
for i in Q:
    let ev = events[i.ord]
    let key = (ev.span.l, ev.span.r)
    match ev.kind:
        Kind.Add =>
            if not active.contains(key): active[key] = []
            active[key].push(i.ord)
        Kind.Remove =>
            let start = active[key].pop()!
            if start < i.ord: add_span(1, 0, q - 1, start, i.ord - 1, ev.span)

for (key, starts) in active:
    let sp = Span(key.0, key.1)
    for start in starts: add_span(1, 0, q - 1, start, q - 1, sp)

var bits: [0..2, 0..m + 2]Int = 0
var log: List<Undo> = []

fn insert_pair(side: 0..2, l: Int, r: Int, cur: ?Int) -> ?Int:
    var hi = 0
    var i = l
    while i > 0:
        hi max= bits[side, i]
        i -= i & -i
    var best = cur
    if hi > 0: best min= r - hi
    var j = r
    while j <= m:
        if l > bits[side, j]:
            log.push(Undo(side, j, bits[side, j]))
            bits[side, j] = l
        j += j & -j
    best

fn better_min_r(cur: ?Span, sp: Span) -> Span =
    if let b = cur:
        if sp.r < b.r or (sp.r == b.r and sp.l > b.l): sp else: b
    else:
        sp

fn better_max_l(cur: ?Span, sp: Span) -> Span =
    if let b = cur:
        if sp.l > b.l or (sp.l == b.l and sp.r < b.r): sp else: b
    else:
        sp

var ans: [Q]?Int = none

fn dfs(o: Int, l: Int, r: Int, init: State):
    let mark = log.len
    var st = init
    for sp in seg[o]:
        let d1 = insert_pair(0, sp.l, sp.r, st.disjoint)
        let d2 = insert_pair(1, m - sp.r, m - sp.l, d1)
        var ml = st.min_len
        ml min= sp.r - sp.l
        st = State(d2, some(better_min_r(st.min_r, sp)), some(better_max_l(st.max_l, sp)), ml)
    if l == r:
        var best_inter = st.min_len!
        var best_hull = st.min_len!
        let left_end = st.min_r!
        let right_start = st.max_l!
        if left_end.r >= right_start.l:
            let inter = left_end.r - right_start.l
            let hull = right_start.r - left_end.l
            if inter < best_inter or (inter == best_inter and hull < best_hull):
                best_inter = inter
                best_hull = hull
        if let d = st.disjoint:
            if 0 < best_inter or (0 == best_inter and d < best_hull):
                best_hull = d
        ans[Q(l)] = best_hull
    else:
        let mid = (l + r) / 2
        dfs(o * 2, l, mid, st)
        dfs(o * 2 + 1, mid + 1, r, st)
    while log.len > mark:
        let u = log.pop()!
        bits[u.side, u.pos] = u.prev

dfs(1, 0, q - 1, State(none, none, none, none))
for i in Q: print(ans[i]!)
