// cco21p5 Bread First Search: low[v] is v's smallest earlier neighbour, or v itself. dp[c], the
// fewest edges to add when the vertices after cut c still have to be placed, picks the next
// block; each later vertex whose low is still too small costs one, kept as range adds over the
// cuts on a min segment tree. The block may run up to the first cut after which every low is
// at least v, found by binary search on the suffix minimum.
let (n, m) = read<(2..=200_000, 0..=200_000)>()
domain V = 1..=n
var low: [V]V = [V] v => v
for (u, v) in read<[m](V, V)>():
    low[max(u, v)] min= min(u, v)
var opened: [V]List<V> = []
for v in V if low[v] != v:
    opened[low[v]].push(v)
let suffix = low.rev.scan(min)
var dp = SegTree(min, [V.cuts] c => if c == V.cuts.last!: 0 else: inf, act: add)
for v in V.drop(1).rev:
    for a in opened[v]:
        dp.apply(a.r.., 1)
    var cur = dp.fold(suffix.partition_point(s => s < v)..)
    if low[v] == v:
        cur += 1
        dp.apply(v.r.., 1)
    dp.set(v.l, cur)
print(dp.get(V.first!.r))
