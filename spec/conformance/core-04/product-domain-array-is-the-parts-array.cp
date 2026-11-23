// expect: ok, output "7 4 3"
// rule: 04.2.11
// `domain Cell = (Row, Col)` makes `[Cell]T` the same type as `[Row, Col]T` (decision 16.5).
domain Row = 0..2
domain Col = 0..3
domain Cell = (Row, Col)
fn total(g: [Row, Col]Int) -> Int = g.sum()
var u: [Cell]Int = 0
u[Row(1), Col(2)] = 4
let c: Cell = (Row(0), Col(1))
u[c] = 3
let t: [Row, Col]Int = u.clone()
print(total(u), t[Row(1), Col(2)], t[c])
