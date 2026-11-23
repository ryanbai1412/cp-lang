// expect: ok, output "-80\n0..3 5 2\n1..2 3 2\n1..3 3 2\n2 1"
// input: "3\n10 100 10\n"
// AtCoder DP L: `[I.ranges]Int` holds one cell per non-empty range, `for r in I.ranges` goes by
// length then left end, and `r.drop(1)` / `r.drop_last(1)` are the inner pieces.
let n = read<Int>()
domain I = 0..n
let a = read<[I]Int>()
var lead: [I.ranges]Int = 0
for r in I.ranges:
    lead[r] = if r.len == 1: a[r.first!] else: max(a[r.first!] - lead[r.drop(1)], a[r.last!] - lead[r.drop_last(1)])
print(lead[I.range])
domain V = 0..4
var cost: [V.ranges]Int = 0
var seen = 0
for r in V.ranges if r.len >= 2:
    cost[r] = r.splits.map((x, y) => cost[x] + cost[y]).min()! + r.len
    seen += 1
print(V.range.take(3), cost[V.range.take(3)], seen - 4)
for (x, y) in V.range.drop(1).splits: print(x, x.len + y.len, cost[(1..3)] + 0 * y.len)
print(cost[V.range.take_last(2)], V.range.drop_last(3).len)
