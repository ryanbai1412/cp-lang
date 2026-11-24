// Source: Baltic OI 2017 Day 2 P2 (Plus Minus), https://dmoj.ca/problem/btoi17p5
// limits: 1000 ms, 1024 MB
// Problem: an N x M grid (N, M <= 1e9) of + and - where every 2 x 2 square holds two of each; K
//   <= 1e5 distinct cells are measured. Count the consistent grids mod 1e9 + 7.
// Solution: such a grid has every row alternating or every column alternating (both: the two
//   checkerboards). With alternating rows a row is set by one cell's sign xor its column parity:
//   rows with measured cells must agree, the rest are free, 2^(free rows) (a Map per row).
//   Columns likewise; subtract the checkerboards counted twice (0, 1 or 2). O(K log K).
// original: corpus/submissions/4456190.cpp
// Bounds: n, m: 1..=1e9; k: 0..=1e5; cells.row: 1..=n; cells.col: 1..=m
type Z = Mod<1000000007>
let n: 1..=1e9 = read()
let m: 1..=1e9 = read()
let k: 0..=1e5 = read()
let cells: [0..k](sign: Char, row: 1..=n, col: 1..=m) = read()

// Ways when every line alternates; each pin is (line, is its line's first
// cell a plus?), and `lines` is how many lines the grid has.
fn alternating(lines: Int, pins: List<(Int, Bool)>) -> Z:
    var seen = Map<Int, Bool>()
    for (line, starts_plus) in pins:
        if let was = seen.get(line):
            if was != starts_plus: return Z.zero
        else: seen[line] = starts_plus
    Z(2) ** (lines - seen.len)

fn odd(x: Int) -> Bool = x % 2 == 1

let by_row = alternating(n, [(f.row, (f.sign == '+') != odd(f.col)) for f in cells])
let by_col = alternating(m, [(f.col, (f.sign == '+') != odd(f.row)) for f in cells])
// Checkerboards are counted on both sides: all cells share one "corner sign".
let boards = alternating(1, [(0, (f.sign == '+') != odd(f.row + f.col)) for f in cells])
print(by_row + by_col - boards)
