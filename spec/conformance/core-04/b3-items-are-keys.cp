// expect: ok, output "60\n0"
// rule: 04a.3.3 04a.9.7 04a.11.2
let a = [1, 2, 3, 4, 5]
var best = [0, 0, 0, 0, 0]
for (i, x) in a[1..=3].items: best[i] = x * 10
print(best[1] + best[3])
print(best[0])
