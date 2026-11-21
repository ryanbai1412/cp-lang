// Codeforces 2207 F: Hanabi.
@!multitest
let (n, m): (Int, Int) = read()
let len = n * m
domain Rank = 1..=n
domain Color = 1..=m
let ranks: [0..len]Rank = read()
let colors: [0..len]Color = read()

var must_rank: [Rank]Bool = false
var run: [Color]?(Rank, Rank) = none
var by_right: [Rank]List<Rank> = []
var total_intervals = 0

for i in 0..len:
    let (r, c) = (ranks[i], colors[i])
    if let (lo, hi) = run[c]:
        if r <= hi: continue
        if r - hi == 1:
            run[c] = (lo, r)
        else:
            if let next = hi + 1:
                for j in next..r: must_rank[j] = true
            by_right[hi].push(lo)
            total_intervals += 1
            run[c] = (r, r)
    else:
        for j in ..r: must_rank[j] = true
        run[c] = (r, r)

for c in Color:
    let (lo, hi) = run[c]!
    if let next = hi + 1:
        for j in next..: must_rank[j] = true
    by_right[hi].push(lo)
    total_intervals += 1

domain Pos = 0..=n
var seg = LazySegTree<Pos, Int>(AddMax, fill: 0)
for r in Rank:
    let v = seg.all + Int(not must_rank[r])
    seg.set(Pos(r.to_idx + 1), v)
    for lo in by_right[r]: seg.apply(..=Pos(lo.to_idx), 1)

print(n + total_intervals - seg.all - 1)
