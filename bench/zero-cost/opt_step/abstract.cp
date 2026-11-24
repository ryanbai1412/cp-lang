let h: 1..=1000 = read()
let w: 1..=1000 = read()
let moves: Str = read()
domain Row = 0..h
domain Col = 0..w
type Cell = (Row, Col)
fn step(p: Cell, move: Char) -> ?Cell =
    if move == '<': Col.at(p.1 - 1).map(c => (p.0, c))
    elif move == '>': Col.at(p.1 + 1).map(c => (p.0, c))
    elif move == '^': Row.at(p.0 - 1).map(r => (r, p.1))
    else: Row.at(p.0 + 1).map(r => (r, p.1))
var total = 0
for b in 0..200:
    let box: Cell = (Row(b % h), Col(b * 7 % w))
    var at: Cell = (Row.first!, Col.first!)
    for move in moves:
        if let next = step(at, move):
            if next != box: at = next
    total += at.0.idx + at.1.idx
print(total)
