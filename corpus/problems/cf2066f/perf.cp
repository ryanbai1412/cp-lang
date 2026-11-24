// Codeforces 2066 F: Curse, tuned: port.cp with the two O(m^2) and O(nm) tables
// narrowed to what they hold (a max subarray sum of b fits in 32 bits, an agreement
// length in 16), as orig.cpp's `int` tensors, and `came` holding only a start cut
// (4 bytes, not a 2-field struct) with `spent` as its outer axis, so each threshold's
// fresh table is half the bytes and a row of b cuts is contiguous. The one transition
// that spends the free segment is marked in `freed`.
@!multitest
use std.algebra.{MaxSub}
domain PosA = read(1..=500)
domain PosB = read(1..=500)
let a: [PosA]-1e6..=1e6 = read()
let b: [PosB]-1e6..=1e6 = read()

fn widest_max(r: Range<PosA>) -> Range<PosA>:
    let f = r.first!
    var best = (a[f], 1, f.l..f.r)
    var (start, run) = (r.start, 0)
    for i in r:
        if run < 0: (start, run) = (i.l, 0)
        run += a[i]
        let key = (run, i.r.ord - start.ord, start..i.r)
        if (key.0, key.1) >= (best.0, best.1): best = key
    best.2

fn cool(r: Range<PosA>) -> List<Range<PosA>>:
    if r.is_empty: return []
    let m = widest_max(r)
    cool(r.start..m.start) + [m] + cool(m.end..r.end)

let spans = cool(PosA.first!..=PosA.last!)
domain Seg = 0..spans.len
let span = [Seg] s => spans[s.ord]
let total = [Seg] s => a[span[s]].sum()

var mss: [PosB.cuts, PosB.cuts]-500_000_000..=500_000_000 = 0
for l in PosB:
    var acc = MaxSub.zero
    for r in l..=PosB.last!:
        acc = acc + MaxSub.of(b[r])
        mss[l.l, r.r] = acc.best

// agree[i, j]: how long a from cut i and b from cut j run equal.
var agree: [PosA.cuts, PosB.cuts]0..=500 = 0
for p in PosA.rev:
    for q in PosB.rev:
        if a[p] == b[q]: agree[p.l, q.l] = agree[p.r, q.r] + 1

fn matches(j: PosB.cuts, s: Seg) -> ?PosB.cuts =
    if agree[span[s].start, j] >= span[s].len: Some(PosB.cuts.nth(j.ord + span[s].len)) else: None

struct Plan(piece: [Seg](from: PosB.cuts, to: PosB.cuts), free: Seg)

fn plan(x: Int) -> ?Plan:
    // came[spent, k, j]: where the piece ending at (k, j) started; freed[k, j]: that
    // start had not spent the free segment yet, though (spent, k, j) has.
    var came: [Bool, Seg.cuts, PosB.cuts]?PosB.cuts = None
    var freed: [Seg.cuts, PosB.cuts]Bool = false
    fn fit(k: Seg.cuts, j: PosB.cuts, spent: Bool) -> Bool =
        came[spent, k, j] != None or (k == Seg.cuts.first and j == PosB.cuts.first and not spent)
    for s in Seg:
        if total[s] < x:
            for spent in [false, true]:
                for j in PosB.cuts:
                    if let k = matches(j, s) and fit(s.l, j, spent):
                        came[spent, s.r, k] = Some(j)
        else:
            for spent in [false, true]:
                var latest: ?PosB.cuts = None
                for j in PosB.cuts:
                    if let i = latest and mss[i, j] <= x: came[spent, s.r, j] = Some(i)
                    if fit(s.l, j, spent): latest = j
            if let i = PosB.cuts.find(j => fit(s.l, j, false)):
                for j in PosB.cuts:
                    if j > i: (came[true, s.r, j], freed[s.r, j]) = (Some(i), true)
    var (at, spent) = (PosB.cuts.last!, true)
    if not fit(Seg.cuts.last!, at, true): return None
    var piece: [Seg](from: PosB.cuts, to: PosB.cuts) = (
        from: PosB.cuts.first!,
        to: PosB.cuts.first!,
    )
    var free: ?Seg = None
    for s in Seg.rev:
        let from = came[spent, s.r, at]!
        piece[s] = (from: from, to: at)
        if spent and freed[s.r, at]: (free, spent) = (Some(s), false)
        at = from
    Plan(piece, free!)

let thresholds = total.sorted(by: x => desc(x)).distinct()
fn first_plan() -> ?(Int, Plan):
    for t in thresholds:
        if let p = plan(t): return Some((t, p))
    None

let (x, plan_x) = first_plan() else:
    print(-1)
    return

var len = [Seg] s => span[s].len
fn start(s: Seg) -> Int = sum(len[t] for t in Seg if t < s)
var ops: List<(Int, Int, List<Int>)> = []
for s in Seg.filter(s => total[s] >= x).sorted(by: s => desc(total[s])):
    let into = if s == plan_x.free: [x] else: b[plan_x.piece[s].from..plan_x.piece[s].to].to_list()
    ops.push((start(s) + 1, start(s) + len[s], into))
    len[s] = into.len
ops.push(
    (
        start(plan_x.free) + 1,
        start(plan_x.free) + 1,
        b[plan_x.piece[plan_x.free].from..plan_x.piece[plan_x.free].to].to_list(),
    )
)

print(ops.len)
for (l, r, into) in ops:
    print(l, r, into.len)
    print(into)
