// Codeforces 2066 F: Curse (dream).
// The cool partition splits a at its widest maximum-sum segment and recurses on
// the two sides. An operation only ever replaces one cool segment whole, and never
// raises the sum the next operation may pick. So b is reachable iff, for some
// threshold x among the segment sums, b is the cool segments in order where
//   - a segment with sum < x stays as it is,
//   - a segment with sum >= x becomes a non-empty array with max subarray sum <= x,
//   - except at most one, the free segment, which becomes anything.
// For each threshold, largest first, fill
//   fit[k, j, spent] = segments before cut k can make b[..j], free segment spent or not.
// A tame piece that may start at cut j may start at any later cut too (a subarray's
// max subarray sum is no larger), so the latest reachable start is the only one to try.
//
// Output: replace the segments with sum >= x in decreasing sum order, the free one
// by [x], then replace [x] by its piece. Each step replaces a maximum-sum segment.
@!multitest
domain PosA = read(1..=500)
domain PosB = read(1..=500)
let a: [PosA]-1e6..=1e6 = read()
let b: [PosB]-1e6..=1e6 = read()

fn cool(r: Range<PosA.Cut>) -> List<Range<PosA.Cut>> =
    if r.is_empty: []
    else:
        let m = a[r].max_subarray(widest: true)
        cool(r.start..m.lo) + [m] + cool(m.hi..r.end)

let (domain Seg, span) = cool(PosA.Cut.first..PosA.Cut.last).indexed()
let sum = [Seg] s => a[span[s]].sum()
let mss = [PosB.Cut, PosB.Cut] (l, r) => b[l..r].max_subarray_sum()

struct Plan(piece: [Seg]Range<PosB.Cut>, free: ?Seg)

fn plan(x: Int) -> ?Plan:
    var fit: [Seg.Cut, PosB.Cut, Bool]Bool = false
    var came: [Seg, PosB.Cut, Bool]?(PosB.Cut, Bool) = none
    fit[Seg.Cut.first, PosB.Cut.first, false] = true
    fn link(s: Seg, j: PosB.Cut, spent: Bool, from: (PosB.Cut, Bool)):
        fit[s.r, j, spent] = true
        came[s, j, spent] = from
    for s in Seg:
        for spent in Bool if sum[s] < x:
            for j in PosB.Cut if fit[s.l, j, spent] and b[j..].starts_with(a[span[s]]):
                link(s, j + span[s].len, spent, from: (j, spent))
        for spent in Bool if sum[s] >= x:
            var latest: ?PosB.Cut = none
            for j in PosB.Cut:
                if let i = latest and mss[i, j]! <= x: link(s, j, spent, from: (i, spent))
                if fit[s.l, j, spent]: latest = j
        if let i = PosB.Cut.first(j => fit[s.l, j, false]) and sum[s] >= x:
            for j in PosB.Cut if j > i: link(s, j, true, from: (i, false))

    var (j, spent) = (PosB.Cut.last, [true, false].first(u => fit[Seg.Cut.last, PosB.Cut.last, u])?)
    var piece: [Seg]Range<PosB.Cut> = PosB.Cut.first..PosB.Cut.first
    var free: ?Seg = none
    for s in Seg.rev:
        let (i, before) = came[s, j, spent]!
        piece[s] = i..j
        if before != spent: free = s
        (j, spent) = (i, before)
    Plan(piece, free)

let thresholds = sum.values.distinct().sort(by: x => -x)
let (x, p) = thresholds.first_some(x => (x, plan(x)?)) else:
    print(-1)
    return

var len = [Seg] s => span[s].len
let start = s => sum(len[t] for t in Seg.before(s))
var ops: List<(Int, Int, List<Int>)> = []
for s in Seg.filter(s => sum[s] >= x).sort(by: s => -sum[s]):
    let into = if p.free == some(s): [x] else: b[p.piece[s]].to_list()
    ops.push((start(s) + 1, start(s) + len[s], into))
    len[s] = into.len
if let s = p.free:
    ops.push((start(s) + 1, start(s) + 1, b[p.piece[s]].to_list()))

print(ops.len)
for (l, r, into) in ops:
    print(l, r, into.len)
    print(into)
