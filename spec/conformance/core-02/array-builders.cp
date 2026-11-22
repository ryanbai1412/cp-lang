// expect: ok, output "4 7 2"
// rule: 02.8.3
domain V = 0..3
let a = [V] v => Int(v) * 2
let b: [V]Int = [V].filled(7)
let c: [V]Int = [Int(v) + 1 for v in V]
print(a[V(2)], b[V(0)], c[V(1)])
