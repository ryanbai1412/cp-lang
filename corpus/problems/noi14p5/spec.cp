// noi14p5 Random Number Generator: build the board from the generator, then take values smallest
// first; a cell (r, c) fits the monotone path iff every row above stays at or left of c and every
// row below at or right of it.
let (x0, a, b, c, d) = read<(
    0..=100_000_000,
    0..=300,
    0..=100_000_000,
    0..=100_000_000,
    1..=100_000_000,
)>()
let (n, m, q) = read<(1..=5000, 1..=5000, 0..=50_000)>()
domain Row = 1..=n
domain Col = 1..=m
domain Grid = (Row, Col)
domain Cell = 1..=n * m // a grid cell numbered in reading order, as the swaps name it
domain Value = 1..=n * m
var board: [Cell]Value = [Cell] p => Value.nth(p.ord)
var x = x0
for p in Cell:
    x = (a * x * x + b * x + c) % d
    board.swap(p, Cell.nth(x % (p.ord + 1)))
for (u, v) in read<[q](Cell, Cell)>():
    board.swap(u, v)
let at = board.inverse()
var lo: [Row]Col = Col.first!
var hi: [Row]Col = Col.last!
var path: List<Value> = []
for v in Value:
    let (r, col) = Grid.nth(at[v].ord)
    if lo[r] <= col <= hi[r]:
        path.push(v)
        for above in r.before:
            hi[above] min= col
        for below in r.after:
            lo[below] max= col
print(path)
