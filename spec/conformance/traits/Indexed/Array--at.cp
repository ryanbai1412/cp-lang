// expect: ok, output "10"
domain V = 0..3
var a = [V] v => Int(v) * 10
print(a[V(1)])
