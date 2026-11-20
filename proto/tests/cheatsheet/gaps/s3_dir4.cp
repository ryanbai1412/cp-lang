let (h, w): (1..=5, 1..=5) = read()
domain Row = 0..h
domain Col = 0..w
type Cell = (Row, Col)
var g: [Row, Col]Char = read()
let p: Cell = (Row(0), Col(1))
let d = Dir4.from("UDLR", 'R')
print(p.step(d), d.cw == d.ccw.back, sum(1 for q in p.ray(d)))
