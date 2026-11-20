// CCO '20 P5 — Interval Collection
// original: corpus/submissions/4562195.cpp
//
// Maintain a dynamic multiset of intervals [l, r]: minimize intersection length
// first, then union length, across all pairs. Disjoint pairs (0 intersection)
// are answered offline via a segment tree over time + two mirrored rollback
// Fenwick trees; overlapping pairs reduce to the min-right / max-left endpoints
// maintained with lazy-deletion heaps.

let q: 1..=500_000 = read()
domain Q = 0..q
let MAXC = 1_000_001

struct Event:
    kind: Char
    l: Int
    r: Int

let ev: [Q]Event = [Q] _ => Event(read(), read(), read())

// Map each interval [l, r] to its active time windows via a LIFO stack per key.
var open = HashMap<(Int, Int), List<Int>>()
var paired: [Q]Int = 0
var seg: List<List<(Int, Int)>> = [[] ; 4 * q]

fn add_span(o: Int, l: Int, r: Int, ql: Int, qr: Int, iv: (Int, Int)):
    if ql <= l and r <= qr:
        seg[o].push(iv)
        return
    let mid = (l + r) / 2
    if ql <= mid: add_span(o * 2, l, mid, ql, qr, iv)
    if mid < qr: add_span(o * 2 + 1, mid + 1, r, ql, qr, iv)

for i in 0..q:
    let e = ev[Q(i)]
    if e.kind == 'A':
        open[(e.l, e.r)].push(i)
    else:
        let start = open[(e.l, e.r)].pop()!
        paired[Q(i)] = start
        add_span(1, 0, q - 1, start, i - 1, (e.l, e.r))
for (iv, starts) in open:
    for s in starts: add_span(1, 0, q - 1, s, q - 1, iv)

struct RollbackBit:
    v: List<Int>
    log: List<(Int, List<Int>)>
    best: Int
    best_log: List<Int>

    fn insert(l: Int, r: Int):
        best_log.push(best)
        // Best prior interval ending at <= l has maximum start stored in bit[1..=l].
        var hi = 0
        var i = l
        while i > 0:
            hi max= v[i]
            i -= i & -i
        if hi > 0: best min= r - hi
        var saved: List<Int> = []
        var j = r
        while j <= MAXC:
            saved.push(v[j])
            v[j] max= l
            j += j & -j
        log.push((r, saved))

    fn rollback():
        let (r0, saved) = log.pop()!
        var j = r0
        for old in saved:
            v[j] = old
            j += j & -j
        best = best_log.pop()!

let INF = 1_000_000_000
var fwd = RollbackBit([0; MAXC + 1], [], INF, [])
var rev = RollbackBit([0; MAXC + 1], [], INF, [])
var disjoint_ans: [Q]Int = INF

fn dfs(o: Int, l: Int, r: Int):
    for (cl, cr) in seg[o]:
        fwd.insert(cl, cr)
        rev.insert(MAXC - cr, MAXC - cl)
    if l == r:
        disjoint_ans[Q(l)] = min(fwd.best, rev.best)
    else:
        let mid = (l + r) / 2
        dfs(o * 2, l, mid)
        dfs(o * 2 + 1, mid + 1, r)
    for _ in 0..seg[o].len:
        fwd.rollback()
        rev.rollback()

dfs(1, 0, q - 1)

// Online sweep with lazy-deletion heaps for overlapping pairs and singletons.
var by_r = Heap<(Int, Int, Int)>()   // (r, -l, id) -> smallest r, largest l
var by_l = Heap<(Int, Int, Int)>()   // (-l, r, id) -> largest l, smallest r
var by_len = Heap<(Int, Int)>()      // (r - l, id)
var dead: [Q]Bool = false

for i in 0..q:
    let e = ev[Q(i)]
    if e.kind == 'A':
        by_r.push((e.r, -e.l, i))
        by_l.push((-e.l, e.r, i))
        by_len.push((e.r - e.l, i))
    else:
        dead[Q(paired[Q(i)])] = true
    while dead[Q(by_r.top()!.2)]: by_r.pop()
    while dead[Q(by_l.top()!.2)]: by_l.pop()
    while dead[Q(by_len.top()!.1)]: by_len.pop()

    let (len0, _) = by_len.top()!
    var ans = (len0, len0)
    let (r1, nl1, _) = by_r.top()!
    let (nl2, r2, _) = by_l.top()!
    let (l1, l2) = (-nl1, -nl2)
    if r1 >= l2:
        ans min= (r1 - l2, r2 - l1)
    let dj = disjoint_ans[Q(i)]
    if dj < INF:
        ans min= (0, dj)
    print(ans.1)
