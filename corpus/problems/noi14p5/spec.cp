// noi14p5 Random Number Generator: build the board from the generator, then take values smallest
// first; a cell (r, c) fits the monotone path iff every row above stays at or left of c and every
// row below at or right of it.
let (x0, a, b, c, d) = read<(0..=100_000_000, 0..=300, 0..=100_000_000, 0..=100_000_000, 1..=100_000_000)>()
let (n, m, q) = read<(1..=5000, 1..=5000, 0..=50_000)>()
domain Row = 1..=n
domain Col = 1..=m
domain Cell = 1..=n * m
var board: [Cell]Cell = [Cell] i => i
var x = x0
for i in Cell:
    x = (a * x * x + b * x + c) % d
    board.swap_at(i, Cell(x % i + 1))
for _ in 0..q:
    let (u, v) = read<(Cell, Cell)>()
    board.swap_at(u, v)
let at = board.inverse()
var lo: [Row]Col = Col.first!
var hi: [Row]Col = Col.last!
var path: List<Cell> = []
for v in Cell:
    let r = Row((at[v] - 1) / m + 1)
    let col = Col((at[v] - 1) % m + 1)
    if lo[r] <= col and col <= hi[r]:
        path.push(v)
        for above in ..r:
            hi[above] = min(hi[above], col)
        for below in r.next..:
            lo[below] = max(lo[below], col)
print(path)
