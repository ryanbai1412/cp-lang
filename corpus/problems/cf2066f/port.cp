// Source: Codeforces 2066F (Curse), https://codeforces.com/contest/2066/problem/F
// Problem: multitest, t <= 200. Arrays a and b (n, m <= 500, sums over tests <= 500, |values|
//   <= 1e6). An operation picks any maximum-sum non-empty subarray of a and replaces it with any
//   non-empty array. Print -1 if b is unreachable, else the operations, with replacements of
//   total length <= n + m and values within 1e9.
// Solution: the cool partition splits a at its widest max-sum segment and recurses on both sides;
//   operations replace cool segments whole. b is reachable iff for some threshold x among the
//   segment sums, b is the segments in order: sum < x kept, sum >= x replaced by a non-empty array
//   with max subarray sum <= x, except one free segment that becomes anything. For each x,
//   largest first, a DP came[segment cut, b cut, free used] matches the pieces; a tame piece
//   only needs the latest reachable start. Replay in decreasing sum, the free one by [x] and
//   then by its piece. O(n^2 m).
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

var mss: [PosB.cuts, PosB.cuts]Int = 0
for l in PosB:
    var acc = MaxSub.zero
    for r in l..=PosB.last!:
        acc = acc + MaxSub.of(b[r])
        mss[l.l, r.r] = acc.best

// agree[i, j]: how long a from cut i and b from cut j run equal.
var agree: [PosA.cuts, PosB.cuts]Int = 0
for p in PosA.rev:
    for q in PosB.rev:
        if a[p] == b[q]: agree[p.l, q.l] = agree[p.r, q.r] + 1

fn matches(j: PosB.cuts, s: Seg) -> ?PosB.cuts =
    if agree[span[s].start, j] >= span[s].len: Some(PosB.cuts.nth(j.ord + span[s].len)) else: None

struct Back(from: PosB.cuts, spent: Bool)
struct Plan(piece: [Seg](from: PosB.cuts, to: PosB.cuts), free: Seg)

fn plan(x: Int) -> ?Plan:
    var came: [Seg.cuts, PosB.cuts, Bool]?Back = None
    fn fit(k: Seg.cuts, j: PosB.cuts, spent: Bool) -> Bool =
        came[k, j, spent] != None or (k == Seg.cuts.first and j == PosB.cuts.first and not spent)
    for s in Seg:
        if total[s] < x:
            for spent in [false, true]:
                for j in PosB.cuts:
                    if let k = matches(j, s) and fit(s.l, j, spent):
                        came[s.r, k, spent] = Back(j, spent)
        else:
            for spent in [false, true]:
                var latest: ?PosB.cuts = None
                for j in PosB.cuts:
                    if let i = latest and mss[i, j] <= x: came[s.r, j, spent] = Back(i, spent)
                    if fit(s.l, j, spent): latest = j
            if let i = PosB.cuts.find(j => fit(s.l, j, false)):
                for j in PosB.cuts:
                    if j > i: came[s.r, j, true] = Back(i, false)
    var at = Back(PosB.cuts.last!, true)
    if not fit(Seg.cuts.last!, at.from, true): return None
    var piece: [Seg](from: PosB.cuts, to: PosB.cuts) = (
        from: PosB.cuts.first!,
        to: PosB.cuts.first!,
    )
    var free: ?Seg = None
    for s in Seg.rev:
        let back = came[s.r, at.from, at.spent]!
        piece[s] = (from: back.from, to: at.from)
        if back.spent != at.spent: free = s
        at = back
    Plan(piece, free!)

let thresholds = total.sorted(desc: true).distinct()
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
