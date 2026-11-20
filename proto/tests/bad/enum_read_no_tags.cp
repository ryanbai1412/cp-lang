// error: needs to know how its variants are written in the input
enum Tile:
    X
    O
let n: 1..=3 = read()
domain Row = 1..=n
let g: [Row, Row]Tile = read()
print(count(g[r, c] == X for r in Row for c in Row))
