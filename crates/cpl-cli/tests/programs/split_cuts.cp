// A user fn shaped like `splits`: its cut walks `r.drop(1)`, so both pieces
// are non-empty and their cells are read with no empty check.
fn halves<D>(r: Range<D>) -> yields (Range<D>, Range<D>):
    for p in r.drop(1):
        let m = (p..p).l
        yield (r.l..m, m..r.r)

let n = 7
domain I = 0..n
var cost: [I.ranges]Int = 0
for r in I.ranges:
    if r.len >= 2:
        cost[r] = min(cost[x] + cost[y] + r.len for (x, y) in halves(r))!
print(cost[I.range])
var cuts = 0
for (x, y) in halves(I.range):
    cuts = cuts * 10 + x.len - y.len + 7
print(cuts)
