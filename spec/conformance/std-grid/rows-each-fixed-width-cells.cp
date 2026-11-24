// expect: ok, output "0\n2\n1 1 1"
// The rows of a `[R, C]T` over a fixed-width `C` are `[C]T` cells: each row reaches its consumer
// as the array the consumer takes, empty or not.
domain R = 0..0
domain C = 0..3
domain S = 0..2
let g: [R, C]Int = 1
let h: [S, C]Int = 1
print(g.rows.each().count())
print(h.rows.each().count())
print(h.rows.each().first()!)
