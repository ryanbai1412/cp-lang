// ioi96p1 A Game: players take a number from either end; both play best. The mover's lead on a
// range is the better end it takes minus the opponent's lead on what is left.
let n: 1..=100 = read()
domain I = 0..n
let a: [I]0..=200 = read()
var lead: [I.ranges]Int = 0
for r in I.ranges:
    lead[r] = if r.len == 1: a[r.first] else: max(a[r.first] - lead[r.drop(1)], a[r.last] - lead[r.drop_last(1)])
let total = a.sum()
print((total + lead[I.range]) / 2, (total - lead[I.range]) / 2)
