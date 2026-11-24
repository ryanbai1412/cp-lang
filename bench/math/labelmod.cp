// Matched benchmark (dmpg18s5): 2e5 windows of 1000 labels, each counted by `a[p].label % x.label == 0`.
const MAX = 200_000
let n = 200_000
domain Pos = 0..n
domain Val = 1..=MAX
let a = [Pos] p => Val(1 + p.ord * 48271 % MAX)
var total = 0
for i in 0..n:
    let x = Val(1 + i * 7919 % MAX)
    let lo = Pos(i * 104729 % (n - 1000))
    total += (lo.label..lo.label + 1000).count(p => a[Pos(p)].label % x.label == 0)
print(total)
