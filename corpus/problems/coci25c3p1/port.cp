// Source: COCI 2025/2026 Round 3 (Kist); not on DMOJ, statement in the COCI archive at hsin.hr,
//   2025_2026/contest3_tasks.pdf
// Problem: an n x n grid of '.' (n odd, n, k <= 50); Ivo starts in the centre cell and runs a
//   word of <= 50 capital letters: L R U D move one cell (skipped if it would leave the grid),
//   any other letter paints every cell at Manhattan distance < k with that letter. Print the grid.
// Solution: simulate: a move happens only when the next cell is in its domain (`Row.at`,
//   `Col.at` give None off the grid), and a paint rewrites every cell within distance k - 1.
//   O(|word| n^2), about 1e5 steps.
// original: corpus/submissions/7730233.cpp
let n: 1..=50 = read()
let k: 1..=50 = read()
let moves: Str = read()
domain Row = 0..n
domain Col = 0..n
var g: [Row, Col]Char = '.'
var r = Row(n / 2)
var c = Col(n / 2)
for ch in moves:
    if ch == 'L':
        if let x = Col.at(c - 1): c = x
    elif ch == 'R':
        if let x = Col.at(c + 1): c = x
    elif ch == 'U':
        if let x = Row.at(r - 1): r = x
    elif ch == 'D':
        if let x = Row.at(r + 1): r = x
    else:
        for i in Row:
            for j in Col:
                if abs(i.ord - r.ord) + abs(j.ord - c.ord) < k: g[i, j] = ch
for i in Row:
    var line = ""
    for j in Col: line.push(g[i, j])
    print(line)
