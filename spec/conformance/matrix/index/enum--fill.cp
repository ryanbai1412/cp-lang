// expect: ok, output "5"
enum D = N | E | S | W
var a: [D]Int = 0
a[D.E] = 5
print(a.sum())
