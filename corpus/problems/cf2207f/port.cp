// Codeforces 2207 F: Hanabi.
// Scan the n * m cards (rank, color) in order. For each color, keep its
// current maximal contiguous increasing run [lo..=hi] (ignoring ranks <= hi
// already covered). Any rank skipped before a color's first card, strictly
// between two consecutive runs of the same color, or after its last run cannot
// be merged across turns (`must_rank[r] = true`).
//
// Each closed run [lo..=hi] is an interval contributing 1 to `total_intervals`.
// Over the cut space `Rank.cuts`, `seg` maximizes the number of saved turns
// (non-forced ranks plus absorbed runs): at rank r, placing a cut at r.r gains
// `Int(not must_rank[r])` over the best previous cut, and every run [lo..=r]
// ending at r adds 1 to all cuts <= lo.l. The minimum number of turns is
// `n + total_intervals - seg.all - 1`.

@!multitest
let (n, m): (1..=200_000, 1..=200_000) = read()
let len = n * m
domain Rank = 1..=n
domain Color = 1..=m
let ranks: [0..len]Rank = read()
let colors: [0..len]Color = read()

var must_rank: [Rank]Bool = false
var run: [Color]?(Rank, Rank) = None
var by_right: [Rank]List<Rank> = []
var total_intervals = 0

fn force_between(after: ?Rank, before: ?Rank):
    let lo: ?Rank = if let a = after: a.next else: Rank.first
    if let s = lo:
        if let b = before:
            for j in s..b: must_rank[j] = true
        else:
            for j in s..=Rank.last!: must_rank[j] = true

fn close_run(lo: Rank, hi: Rank, before: ?Rank):
    force_between(hi, before)
    by_right[hi].push(lo)
    total_intervals += 1

for (r, c) in zip(ranks, colors):
    if let (lo, hi) = run[c]:
        if r <= hi: continue
        if r - hi == 1:
            run[c] = (lo, r)
        else:
            close_run(lo, hi, r)
            run[c] = (r, r)
    else:
        force_between(None, r)
        run[c] = (r, r)

for c in Color:
    let (lo, hi) = run[c]!
    close_run(lo, hi, None)

var seg = SegTree<Rank.cuts, Int>(max, fill: 0, act: add)
for r in Rank:
    let v = seg.all + Int(not must_rank[r])
    seg.set(r.r, v)
    for lo in by_right[r]: seg.apply(..=lo.l, 1)

print(n + total_intervals - seg.all - 1)
