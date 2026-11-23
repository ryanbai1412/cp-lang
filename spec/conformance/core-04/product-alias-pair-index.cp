// expect: ok, output "5 7 12"
// rule: 04.2.10
// `[P, P]T` with `type P = (A, B)` has the four factor axes; each product position stands for its own factors.
domain A = 0..2
domain B = 0..3
type P = (A, B)
var d: [P, P]Int = 0
let p: P = (A(1), B(2))
let q: P = (A(0), B(1))
d[p, p] = 5
d[p, q] += 7
print(d[p, p], d[p, q], d.sum())
