// Codeforces 2207 F: Hanabi.
// Scan the n * m cards (rank, color) in order. For each color, keep its
// current maximal contiguous increasing run [lo..=hi] (ignoring ranks <= hi
// already covered). Any rank skipped before a color's first card, strictly
// between two consecutive runs of the same color, or after its last run cannot
// be merged across turns (`must_rank[r] = true`).
//
// Each closed run [lo..=hi] is an interval contributing 1 to `total_intervals`.
// Over the cut space `Rank.bounds`, `seg` maximizes the number of saved turns
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
var run: [Color]?(Rank, Rank) = none
var by_right: [Rank]List<Rank> = []
var total_intervals = 0

fn close_run(lo: Rank, hi: Rank, before: ?Rank):
    // Rank.last has no successor (hi + 1 == none), so no gap follows it.
    if let next = hi + 1:
        if let r = before:
            for j in next..r: must_rank[j] = true
        else:
            for j in next..: must_rank[j] = true
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
        for j in ..r: must_rank[j] = true
        run[c] = (r, r)

for c in Color:
    let (lo, hi) = run[c]!
    close_run(lo, hi, none)

var seg = LazySegTree<Rank.bounds, Int>(AddMax, fill: 0)
for r in Rank:
    let v = seg.all + Int(not must_rank[r])
    seg.set(r.r, v)
    for lo in by_right[r]: seg.apply(..=lo.l, 1)

print(n + total_intervals - seg.all - 1)
