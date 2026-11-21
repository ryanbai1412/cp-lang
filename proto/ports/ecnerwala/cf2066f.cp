// Codeforces 2066 F: Curse.
// The cool partition splits a at its widest maximum-sum segment and recurses on
// the two sides. An operation only ever replaces one cool segment whole, and never
// raises the sum the next operation may pick. So b is reachable iff, for some
// threshold x among the segment sums, b is the cool segments in order where
//   - a segment with sum < x stays as it is,
//   - a segment with sum >= x becomes a non-empty array with max subarray sum <= x,
//   - except one, the free segment, which becomes anything non-empty.
// For each threshold, largest first, `came[k, j, spent]` is how the segments
// before cut k make b[..j], the free segment spent or not. A tame piece that may
// start at cut j may start at any later cut too (a subarray's max subarray sum is
// no larger), so the latest reachable start is the only one to try.
//
// Output: replace the segments with sum >= x in decreasing sum order, the free one
// by [x], then replace [x] by its piece. Each step replaces a maximum-sum segment.
@!multitest
use std.algebra.{MaxSub}
domain PosA = read(1..=500)
domain PosB = read(1..=500)
let a: [PosA]-1e6..=1e6 = read()
let b: [PosB]-1e6..=1e6 = read()

fn widest_max(r: Range<PosA>) -> Range<PosA>:
    var best: ?(Int, Int, Range<PosA>) = none
    var (start, run) = (r.start, 0)
    for i in r:
        if run < 0: (start, run) = (i.l, 0)
        run += a[i]
        let key = (run, i.r.ord - start.ord, start..i.r)
        if best == none or (key.0, key.1) >= (best!.0, best!.1): best = key
    best!.2

fn cool(r: Range<PosA>) -> List<Range<PosA>>:
    if r.is_empty: return []
    let m = widest_max(r)
    cool(r.start..m.start) + [m] + cool(m.end..r.end)

let spans = cool(PosA.first..=PosA.last)
domain Seg = 0..spans.len
let span = [Seg] s => spans[s.ord]
let total = [Seg] s => a[span[s]].sum()

var mss: [PosB.bounds, PosB.bounds]Int = 0
for l in PosB:
    var acc = MaxSub.empty()
    for r in l..=PosB.last:
        acc = acc + MaxSub.of(b[r])
        mss[l.l, r.r] = acc.best

// agree[i, j]: how long a from cut i and b from cut j run equal.
var agree: [PosA.bounds, PosB.bounds]Int = 0
for p in PosA.rev:
    for q in PosB.rev:
        if a[p] == b[q]: agree[p.l, q.l] = agree[p.r, q.r] + 1

fn matches(j: PosB.bounds, s: Seg) -> ?PosB.bounds =
    if agree[span[s].start, j] >= span[s].len: some(PosB.bounds.nth(j.ord + span[s].len)) else: none

struct Back(from: PosB.bounds, spent: Bool)
struct Plan(piece: [Seg]Range<PosB>, free: Seg)

fn plan(x: Int) -> ?Plan:
    var came: [Seg.bounds, PosB.bounds, Bool]?Back = none
    fn fit(k: Seg.bounds, j: PosB.bounds, spent: Bool) -> Bool =
        came[k, j, spent] != none or (
            k == Seg.bounds.first and j == PosB.bounds.first and not spent
        )
    for s in Seg:
        if total[s] < x:
            for spent in [false, true]:
                for j in PosB.bounds:
                    if let k = matches(j, s) and fit(s.l, j, spent):
                        came[s.r, k, spent] = Back(j, spent)
        else:
            for spent in [false, true]:
                var latest: ?PosB.bounds = none
                for j in PosB.bounds:
                    if let i = latest and mss[i, j] <= x: came[s.r, j, spent] = Back(i, spent)
                    if fit(s.l, j, spent): latest = j
            if let i = PosB.bounds.first(j => fit(s.l, j, false)):
                for j in PosB.bounds:
                    if j > i: came[s.r, j, true] = Back(i, false)
    var at = Back(PosB.bounds.last, true)
    if not fit(Seg.bounds.last, at.from, true): return none
    var piece: [Seg]Range<PosB> = PosB.bounds.first..PosB.bounds.first
    var free: ?Seg = none
    for s in Seg.rev:
        let back = came[s.r, at.from, at.spent]!
        piece[s] = back.from..at.from
        if back.spent != at.spent: free = s
        at = back
    Plan(piece, free!)

let thresholds = [total[s] for s in Seg].sorted(by: t => -t).distinct()
fn first_plan() -> ?(Int, Plan):
    for t in thresholds:
        if let p = plan(t): return some((t, p))
    none

let (x, plan_x) = first_plan() else:
    print(-1)
    return

var len = [Seg] s => span[s].len
fn start(s: Seg) -> Int = sum(len[t] for t in Seg if t < s)
var ops: List<(Int, Int, List<Int>)> = []
for s in Seg.filter(s => total[s] >= x).sorted(by: s => -total[s]):
    let into = if s == plan_x.free: [x] else: b[plan_x.piece[s]].to_list()
    ops.push((start(s) + 1, start(s) + len[s], into))
    len[s] = into.len
ops.push((start(plan_x.free) + 1, start(plan_x.free) + 1, b[plan_x.piece[plan_x.free]].to_list()))

print(ops.len)
for (l, r, into) in ops:
    print(l, r, into.len)
    print(into)
