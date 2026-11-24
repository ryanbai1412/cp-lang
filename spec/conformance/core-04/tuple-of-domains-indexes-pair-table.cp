// expect: ok, output "3"
// rule: 04.2.11
// For P = (A, B), [P]T and [A, B]T are one type: a tuple position indexes a two-axis table whole.
domain V = 0..3
let c: [V, V]Int = [V, V] (i, j) => Int(i) + Int(j)
let p: (V, V) = (V(1), V(2))
print(c[p])
