// `[V.ranges]T` cells in both engines: every key of `V.ranges` has its own
// cell, written by length and read back through `drop`, `drop_last`,
// `splits`, a range literal and the two-end index (lib/search-dp.md).
domain I = 0..6
let a: [I]Int = [3, 1, 4, 1, 5, 9]
var lead: [I.ranges]Int = 0
var cost: [I.ranges]Int = 0
var at: [I.ranges]Int = 0
var k = 0
for r in I.ranges:
    at[r] = k
    k += 1
    lead[r] = if r.len == 1: a[r.first] else: max(
        a[r.first] - lead[r.drop(1)],
        a[r.last] - lead[r.drop_last(1)],
    )
    if r.len >= 2:
        cost[r] = min(cost[x] + cost[y] for (x, y) in r.splits)! + r.len
print(lead[I.range], cost[I.range])
var j = 0
var walk = 0
for r in I.ranges:
    walk = walk * 3 % 1000003 + at[r] * (j + 1)
    j += 1
print(j, walk, at[I.range])
let r = I.range
print(lead[r.drop(2)], lead[r.drop_last(5)], cost[r.drop(1).drop_last(1)])
