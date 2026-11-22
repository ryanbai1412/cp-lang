// expect: ok, output "5"
domain R = 0..2
domain C = 0..3
domain D = (R, C)
var a: [D]Int = 0
a[D((1, 2))] = 5
print(a.sum())
