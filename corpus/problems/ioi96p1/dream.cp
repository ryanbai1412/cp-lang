// IOI 1996 A Game: players alternately take a number from either end; both play optimally.
// The player to move on a range gets its sum minus the best the other gets on what remains.
let n: 1..=100 = read()
domain I = 0..n
let a: [I]0..=200 = read()
let pre = a.scan(+)
var best: [I.ranges]Int = 0
for r in I.ranges:
    best[r] = if r.len == 1: a[r.first] else: pre.fold(r) - min(best[r.drop(1)], best[r.drop_last(1)])
let first = best[I.range]
print(first, a.sum() - first)
