domain Row = 0..2
domain Col = 0..3
domain Cell = (Row, Col)
let ords: [Cell]Int = [Cell] p => p.ord
print([ords[c] for c in Cell])
fn flip<V>() -> [V]Int = [V] v => V.count - 1 - v.ord
let back: [Cell]Int = flip<Cell>()
print([back[c] for c in Cell])
let rows: [Cell]Int = [Cell] p => p.0.idx * 10 + p.1.idx
print([rows[c] for c in Cell])
