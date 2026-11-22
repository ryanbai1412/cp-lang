// expect: ok, output "5"
domain D = 0..4
var a: [D]Int = 0
a[D(1)] = 5
print(a.sum())
