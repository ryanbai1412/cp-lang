// expect: ok, output "5"
domain V = 0..3
var a: [V.cuts]Int = 0
a[V.cuts(1)] = 5
print(a.sum())
