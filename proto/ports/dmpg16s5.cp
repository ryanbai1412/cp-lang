// cp: 0.1 1f59ac0+dirty
// DMPG 2016 S5 Pizza Bag: the tastiest run of at most k consecutive slices
// of a circular pizza. On the pizza unrolled twice, a run ending at slice p
// is a prefix sum minus the smallest prefix sum among the k cuts before it.
let n: 1..=1e5 = read()
let k: 1..=1e5 = read() where k <= n
domain Slice = 0..n
let taste: [Slice]-1e9..=1e9 = read()
domain Pos = 0..2 * n
type Cut = Pos.bounds

var pre: [Cut]Int = 0
for p in Pos: pre[p.r] = pre[p.l] + taste[Slice(p.idx % n)]
var lowest = SegTree<Cut, Int>(inf, (a, b) => min(a, b))
for c in Cut: lowest.set(c, pre[c])

var best = 0
for p in Pos:
    best max= pre[p.r] - lowest.query(Cut(max(p.idx + 1 - k, 0)), p.l)
print(best)
