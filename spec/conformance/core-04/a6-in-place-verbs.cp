// expect: ok, output "234\n32"
// rule: 04a.2.6
var a = [5, 4, 3, 2, 1]
a[1..4].sort()
print(a[1] * 100 + a[2] * 10 + a[3])
a[1..3].reverse()
print(a[1] * 10 + a[2])
