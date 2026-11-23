// cco21p5 Bread First Search: low[v] is v's smallest earlier neighbour, or v itself. dp[c], the
// fewest edges to add when the vertices after cut c still have to be placed, picks the next
// block; each later vertex whose low is still too small costs one, kept as range adds over the
// cuts on a min segment tree. The block may run up to `start`, the first cut after which every
// low is at least v; it only moves left as v falls.
let (n, m) = read<(2..=200_000, 0..=200_000)>()
domain V = 1..=n
var low: [V]V = [V] v => v
for (u, v) in read<[m](V, V)>():
    low[max(u, v)] min= min(u, v)
var opened: [V]List<V> = []
for v in V if low[v] != v:
    opened[low[v]].push(v)
var dp = SegTree(min, [V.cuts] c => if c == V.cuts.last!: 0 else: inf, act: add)
var start = V.cuts.last!
for v in V.drop(1).rev:
    while start.l.is_some_and(u => low[u] >= v):
        start = start.prev!
    for a in opened[v]:
        dp.apply(a.r.., 1)
    var cur = dp.fold(start..)
    if low[v] == v:
        cur += 1
        dp.apply(v.r.., 1)
    dp.set(v.l, cur)
print(dp.get(V.first!.r))
