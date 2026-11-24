// Source: COCI 2025/2026 Round 3 (Kist); not on DMOJ, statement in the COCI archive at hsin.hr,
//   2025_2026/contest3_tasks.pdf
// limits: 1000 ms, 512 MB
// Problem: an n x n grid of '.' (n odd, n, k <= 50); Ivo starts in the centre cell and runs a
//   word of <= 50 capital letters: L R U D move one cell (skipped if it would leave the grid),
//   any other letter paints every cell at Manhattan distance < k with that letter. Print the grid.
// Solution: simulate: a move happens only when the next cell is in its domain (`Row.at`,
//   `Col.at` give None off the grid), and a paint rewrites every cell within distance k - 1.
//   O(|word| n^2), about 1e5 steps.
// original: corpus/submissions/7730233.cpp
// Bounds: n, k: 1..=50

let n: 1..=50 = read()
let k: 1..=50 = read()
let word: Str = read()
domain Row = 0..n
domain Col = 0..n
type Cell = (Row, Col)

// the cell one step from p, None when the letter is no move or the step leaves the grid
fn step(p: Cell, letter: Char) -> ?Cell =
    if letter == 'L': Col.at(p.1 - 1).map(c => (p.0, c))
    elif letter == 'R': Col.at(p.1 + 1).map(c => (p.0, c))
    elif letter == 'U': Row.at(p.0 - 1).map(r => (r, p.1))
    else: Row.at(p.0 + 1).map(r => (r, p.1))

var grid: [Row, Col]Char = '.'
var at: Cell = (Row.nth(n / 2), Col.nth(n / 2))
for letter in word:
    if "LRUD".contains(letter):
        if let next = step(at, letter): at = next
    else:
        for r in Row:
            for c in Col:
                if abs(r - at.0) + abs(c - at.1) < k: grid[r, c] = letter
for r in Row: print("".join(["{grid[r, c]}" for c in Col]))
