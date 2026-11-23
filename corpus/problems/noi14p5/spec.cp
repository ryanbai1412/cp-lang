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
domain Value = 1..=n * m
var board: [Grid]Value = [Grid] p => Value.nth(0)
for (k, p) in Grid.items:
    board[p] = Value.nth(k)
var x = x0
for (k, p) in Grid.items:
    x = (a * x * x + b * x + c) % d
    board.swap(p, Grid.nth(x % (k + 1)))
for (u, v) in read<[q](Value, Value)>():
    board.swap(Grid.nth(Int(u) - 1), Grid.nth(Int(v) - 1))
let at = board.inverse()
var lo: [Row]Col = Col.first!
var hi: [Row]Col = Col.last!
var path: List<Value> = []
for v in Value:
    let (r, col) = at[v]
    if lo[r] <= col <= hi[r]:
        path.push(v)
        for above in ..r:
            hi[above] min= col
        for below in r.next..:
            lo[below] max= col
print(path)
