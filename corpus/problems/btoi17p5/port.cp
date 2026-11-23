// Source: Baltic OI 2017 Day 2 P2 (Plus Minus), https://dmoj.ca/problem/btoi17p5
// Problem: an N x M grid (N, M <= 1e9) of + and - where every 2 x 2 square holds two of each; K
//   <= 1e5 distinct cells are measured. Count the consistent grids mod 1e9 + 7.
// Solution: such a grid has every row alternating or every column alternating (both: the two
//   checkerboards). With alternating rows a row is set by one cell's sign xor its column parity:
//   rows with measured cells must agree, the rest are free, 2^(free rows) (a Map per row).
//   Columns likewise; subtract the checkerboards counted twice (0, 1 or 2). O(K log K).
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
    cells.push(Fixed(plus: sign == '+', row:, col:))

// Ways when every line alternates; each cell gives (line, is its line's
// first cell a plus?). `lines` is how many lines the grid has.
fn alternating(lines: Int, pins: List<(Int, Bool)>) -> Z:
    var seen = Map<Int, Bool>()
    for (line, starts_plus) in pins:
        if let was = seen.get(line):
            if was != starts_plus: return Z.zero
        else:
            seen[line] = starts_plus
    Z(2) ** (lines - seen.len)

fn odd(x: Int) -> Bool = x % 2 == 1

let by_row = alternating(n, [(f.row, f.plus != odd(f.col)) for f in cells])
let by_col = alternating(m, [(f.col, f.plus != odd(f.row)) for f in cells])
// Checkerboards are counted on both sides: all cells share one "corner sign".
let boards = alternating(1, [(0, f.plus != odd(f.row + f.col)) for f in cells])
print(by_row + by_col - boards)
