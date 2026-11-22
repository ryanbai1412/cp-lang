// expect: ok, output "5"
let (domain D, comp) = compress([30, 10, 20, 10])
var a: [D]Int = 0
a[D.last!] = 5
print(a.sum())
