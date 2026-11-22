// expect: ok, output "3 4 4 4"
// rule: 04.2.18
domain V = 2..5
let a: List<V> = [3, 4]
let b: [V]V = [2, 4, 3]
let d: List<(u: V, v: V)> = [(u: 2, v: 4)]
print(a, b[V(3)], d[0].v)
