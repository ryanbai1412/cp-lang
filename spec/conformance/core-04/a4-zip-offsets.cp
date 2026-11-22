// expect: ok, output "11"
// rule: 04a.2.4 04b.1.3
let a = [1, 2, 3, 4]
var s = 0
for (x, y) in zip(a[2..], a[..2]): s += x * y
print(s)
