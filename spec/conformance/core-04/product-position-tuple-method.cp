// expect: ok, output "3"
// rule: 04.2.10
// A product-domain position is a tuple of factor positions, so `(R, C)` methods like `adj4` apply to it.
domain R = 0..2
domain C = 0..3
domain Cell = (R, C)
var g: [Cell]Int = 0
let q: Cell = (R(1), C(1))
for n in q.adj4: g[n] += 1
print(g.sum())
