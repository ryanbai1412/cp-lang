// expect: ok, output "2"
// rule: 04.2.10
// A product position is a tuple of factor positions, so a method on `(R, C)` applies to it.
domain R = 1..=2
domain C = 1..=2
domain Cell = (R, C)
let p = Cell.first!
print(p.adj4.count())
