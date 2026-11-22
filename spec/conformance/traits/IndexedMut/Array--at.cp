// expect: ok, output "7"
domain V = 0..3
var a = [V] v => Int(v) * 10
a[V(1)] = 7
print(a[V(1)])
