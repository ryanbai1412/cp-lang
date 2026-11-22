// cco21p5 Bread First Search: low[v] is v's smallest earlier neighbour. dp[i], the fewest edges
// to add when vertices i.. still have to be placed, picks the next block i..j; each later vertex
// whose low is still too small costs one, kept as range adds on a min segment tree.
let (n, m) = read<(2..=200_000, 0..=200_000)>()
var low: [n]0..=n = n
for (u, v) in read<[m](1..=n, 1..=n)>():
    low[max(u, v) - 1] min= min(u, v) - 1
var opened: [n]List<Int> = []
for i in 0..n:
    if low[i] < n:
        opened[low[i]].push(i)
let suffix = low.rev.scan(min)
var dp = SegTree(min, [0..=n] j => if j == n: 0 else: inf, act: add)
var j = n
for i in (1..n).rev:
    while j >= 0 and suffix[j] >= i:
        j -= 1
    for a in opened[i]:
        dp.apply(a + 1..=n, 1)
    var cur = dp.fold(j + 1..=n)
    if low[i] == n:
        cur += 1
        dp.apply(i + 1..=n, 1)
    dp.set(i, cur)
print(dp.get(1))
