let (h, w): (1..=5, 1..=5) = read()
domain Row = 0..h
domain Col = 0..w
type Cell = (Row, Col)
var g: [Row, Col]Char = read()
let p: Cell = (Row(0), Col(1))
swap(mut g.row(Row(0)), mut g.row(Row(1)))
print(g[p])
