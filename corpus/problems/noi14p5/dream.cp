// NOI 2014 Random Number Generator: generate a permutation grid with the given quadratic
// generator and swaps, then print the lexicographically smallest sorted value list of a
// right/down path from the top-left. Take values smallest first; a value is usable when its cell
// keeps every row's allowed column interval non-empty, and taking it narrows rows above and below.
let (x0, a, b, c, d) = read<(
    0..=100_000_000,
    0..=300,
    0..=100_000_000,
    0..=100_000_000,
    1..=100_000_000,
)>()
let (n, m, q) = read<(1..=5000, 1..=5000, 0..=50_000)>()
domain R = 0..n
domain Col = 0..m
domain Cell = (R, Col)
var x = x0
fn rand() -> Int:
    x = (a * x * x + b * x + c) % d
    x
domain Spot = 0..n * m  // the cells, numbered row by row
domain Val = 0..n * m
var value: [Spot]Val = [Spot] s => Val(s.ord)
for s in Spot: value.swap(s, Spot(rand() % (s.ord + 1)))
for (u, v) in read<[q](1..=n * m, 1..=n * m)>(): value.swap(Spot(u - 1), Spot(v - 1))
let at = value.inverse()

var lo: [R]Col = Col.first!
var hi: [R]Col = Col.last!
var path: List<Int> = []
for (v, s) in at.items:
    let (r, col) = Cell.nth(s.ord)
    if col < lo[r] or col > hi[r]: continue
    path.push(Int(v) + 1)
    for row in R:
        if row < r: hi[row] min= col
        if row > r: lo[row] max= col
print(path)
