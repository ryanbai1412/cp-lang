// expect: ok, output "5"
domain B = 0..3
var a: [Subset<B>]Int = 0
a[Subset<B>.full] = 5
print(a.sum())
