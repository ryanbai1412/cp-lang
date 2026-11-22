// expect: ok, output "10"
// rule: 04.4.5 04.8.5
domain V = 1..=5
let a = [V] v => Int(v)
let i = V.cuts(4)
var best = 0
for j in ..i: best = max(best, a[j..i].sum())
print(best)
