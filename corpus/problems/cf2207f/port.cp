// Source: Codeforces 2207F (Hanabi), https://codeforces.com/contest/2207/problem/F
// limits: 3000 ms, 512 MB
// Problem: Zuko holds all n m cards (rank 1..n, colour 1..m; n m <= 2e5) in a given order. Each
//   turn Iroh clues a rank or a colour (it must highlight a card left) and Zuko plays the leftmost
//   highlighted card; rank r needs rank r - 1 of its colour played first. Print the least number
//   of clue changes that plays every card validly. Multitest, sum n m <= 2e5.
// Solution: per colour, scan its cards in hand order and cut them into runs lo..=hi where each
//   card is the next rank (a lower rank seen later is skipped); a rank a colour jumps over before,
//   between or after its runs is forced (`must_rank`). A DP over the cuts between ranks then
//   maximises the saved clues: cut r scores the best earlier cut plus 1 if rank r is not forced,
//   and each run lo..=r adds 1 to every cut <= lo.l. A max SegTree with range add holds it; the
//   answer is n + runs - best - 1, O(n m + n log n).
// Bounds: n, m: 1..=200_000; ranks: 1..=n; colors: 1..=m

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

for i in 0..len:  // `zip(ranks, colors)` loses the Color type (compiler bug filed)
    let (r, c) = (ranks[i], colors[i])
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
