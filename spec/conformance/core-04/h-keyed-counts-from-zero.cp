// expect: ok, output "40"
// rule: 04a.9.6 04a.11.2
let a = [10, 20, 30, 40]
var s = 0
for (k, x) in a[2..].keyed().items: s += k * x
print(s)
