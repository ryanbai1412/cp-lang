// A domain is the sequence of its positions, so Seq methods apply to it.
domain Row = 0..2
domain Col = 0..3
domain Cell = (Row, Col)
let g: [Cell]Int = [Cell] p => 7
print(Int(Row.find(r => Int(r) == 1)!), Row.count(r => Int(r) >= 1))
print(Cell.count(p => g[p] == 7))
let (r, c) = Cell.find(p => p != Cell.all.first!)!
print(Int(r), Int(c))
