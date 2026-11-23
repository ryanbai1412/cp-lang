let (h, w): (1..=5, 1..=5) = read()
domain Row = 0..h
domain Col = 0..w
type Cell = (Row, Col)
var g: [Row, Col]Char = read()
let p: Cell = (Row(0), Col(1))
print(sum(1 for d in Dir8 if p.step(d) != None))
