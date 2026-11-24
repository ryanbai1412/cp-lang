// expect: ok, output "0 1 2\n3"
// rule: 04b.1.12
// `g.cols` of a one-row grid has one element per column (adversarial round 47).
domain R = 0..1
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => Int(c)
print([col.sum() for col in g.cols])
print(g.rows[R(0)].sum())
