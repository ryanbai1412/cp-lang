// expect: ok, output "11\n4"
// rule: 04.2.10
// `[P] (r, c) => …` and a `(r, c) => …` lambda over `P` name the factors of each position.
domain R = 0..2
domain C = 0..2
domain Cell = (R, C)
let g = [Cell] (r, c) => Int(r) * 10 + Int(c)
print(g[R(1), C(1)])
print(Cell.all.map((r, c) => Int(r) + Int(c)).sum())
