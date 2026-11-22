// NOI 2014 Random Number Generator: generate a permutation grid with the given quadratic
// generator and swaps, then print the lexicographically smallest sorted value list of a
// right/down path from the top-left. Take values smallest first; a value is usable when its cell
// keeps every row's allowed column interval non-empty, and taking it narrows rows above and below.
let (x0, a, b, c, d) = read (Int, Int, Int, Int, Int)
let (n, m, q) = read (Int, Int, Int)
domain R = [n]
domain Col = [m]
domain Cell = (R, Col)
var x = x0
fn rand() -> Int:
    x = (a * x * x + b * x + c) % d
    x
var value: [Cell]Int = [for (i, _) in Cell.items: i]
for (i, cell) in Cell.items: value.swap(cell, Cell.nth(rand() % (i + 1)))
for (u, v) in read [q](Int, Int): value.swap(Cell.nth(u - 1), Cell.nth(v - 1))
let at = value.inverse()

var lo: [R]Col = Col.first!
var hi: [R]Col = Col.last!
var path: List<Int> = []
for (v, (r, col)) in at.items:
    if col < lo[r] or col > hi[r]: continue
    path.push(v + 1)
    for above in ..r: hi[above] min= col
    for below in r<..: lo[below] max= col
print(path)
