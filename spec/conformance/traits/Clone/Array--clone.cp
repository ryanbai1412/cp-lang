// expect: ok, output "1 2 3 9 2 3"
// engines: next
domain V = 0..3
let a: [V]Int = [1, 2, 3]
var b = a.clone()
b[V(0)] = 9
print(a, b)
