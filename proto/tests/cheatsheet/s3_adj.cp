domain Row = 0..3
domain Col = 0..3
type Cell = (Row, Col)
var g: [Row, Col]Int = 0
let corner: Cell = (Row(0), Col(0))
let mid: Cell = (Row(1), Col(1))
g[mid] = 1
print(sum(1 for p in corner.adj4), sum(1 for p in corner.adj8), corner.adj4.len, sum(1 for p in mid.adj8 if g[p] == 0), count(g[p] == 0 for p in mid.adj4))
