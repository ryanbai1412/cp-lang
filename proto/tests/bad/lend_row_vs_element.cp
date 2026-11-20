// error: different projections of the same value
let h: 2..=9 = read()
let w: 1..=9 = read()
domain Row = 1..=h
domain Col = 1..=w
var g: [Row, Col]Char = read()
let r1 = Row(1)
let r2 = Row(2)
let row = g.row(r1)
g[r2, Col.first] = '.'
print(row[Col.first])
