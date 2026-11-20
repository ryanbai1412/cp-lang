// Baltic OI 2017 Plus Minus: an n x m grid of + and - where every 2 x 2
// square holds two of each; k cells are already fixed. Count the fillings
// modulo 1e9 + 7.
// Such a grid has every row alternating or every column alternating (or
// both, the two checkerboards). An alternating row is fixed by one cell, so
// rows without a fixed cell are free and rows with fixed cells must agree.
// original: corpus/submissions/4456190.cpp
type Z = Mod<1000000007>
let n: 1..=1e9 = read()
let m: 1..=1e9 = read()
let k: 0..=100000 = read()

struct Fixed:
    plus: Bool
    row: Int
    col: Int

var cells: List<Fixed> = []
for _ in 0..k:
    let sign: Char = read()
    let row: Int = read()
    let col: Int = read()
    cells.push(Fixed(plus: sign == '+', row: row, col: col))

// Ways when every line alternates; each cell gives (line, is its line's
// first cell a plus?). `lines` is how many lines the grid has.
fn alternating(lines: Int, pins: List<(Int, Bool)>) -> Z:
    var seen = Map<Int, Bool>()
    for (line, starts_plus) in pins:
        if let was = seen.get(line):
            if was != starts_plus: return Z(0)
        else:
            seen[line] = starts_plus
    Z(2) ** (lines - seen.len)

fn odd(x: Int) -> Bool = x % 2 == 1

let by_row = alternating(n, [(f.row, f.plus != odd(f.col)) for f in cells])
let by_col = alternating(m, [(f.col, f.plus != odd(f.row)) for f in cells])
// Checkerboards are counted on both sides: all cells share one "corner sign".
let boards = alternating(1, [(0, f.plus != odd(f.row + f.col)) for f in cells])
print(by_row + by_col - boards)
