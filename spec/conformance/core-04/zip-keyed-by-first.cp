// expect: ok, output "5"
// gap: compiler 2026-11-24T05Z
// rule: 04b.1.3
let a = [1, 2, 3, 4]
let b = [7, 8]
var s = 0
for (k, (x, y)) in zip(a[2..], b).items: s += k
print(s)
