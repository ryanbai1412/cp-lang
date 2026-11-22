// expect: ok, output "5"
domain V = 0..4
var a: [Pair<V>]Int = 0
a[Pair<V>.last!] = 5
print(a.sum())
