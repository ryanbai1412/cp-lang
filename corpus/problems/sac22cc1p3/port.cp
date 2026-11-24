// Source: SAC '22 Code Challenge 1 P3 (That Pool), https://dmoj.ca/problem/sac22cc1p3
// Problem: an N x M grid (N, M, Q <= 100) of 'X' heads, 'W' blood and '.' empty. Action 1
//   drops every X one row (overwriting the cell below; off the bottom it vanishes), then blood
//   alternates: every W moves to the leftmost cell it can reach in its row, then every W with
//   an empty cell below drops one row, until nothing moves. Action 2 prints the grid.
// Solution: plain simulation. Drop X rows bottom-up, then repeat until stable: slide each W
//   left through empty cells (row by row, left to right, so each W stops at its leftmost
//   reachable cell), then collect every W with an empty cell below and drop them all at once.
//   Each round moves blood down or left, so a step costs O((NM)^2) at worst.
// original: corpus/submissions/3992599.cpp
// Bounds: n, m, q: 1..=50
let n: 1..=50 = read()
let m: 1..=50 = read()
domain Row = 0..n
domain Col = 0..m
var g: [Row, Col]Char = read()
let q: 1..=50 = read()

fn run_left() -> Bool:
    var changed = false
    for r in Row:
        for c in Col:
            if g[r, c] != 'W': continue
            var at = c
            while let l = Col.at(at - 1):
                if g[r, l] != '.': break
                g[r, l] = 'W'
                g[r, at] = '.'
                at = l
                changed = true
    changed

// Every drop falls on the board as it stood before this pass.
fn fall() -> Bool:
    var moves: List<(Row, Col)> = []
    for r in Row:
        for c in Col:
            if let below = Row.at(r + 1):
                if g[r, c] == 'W' and g[below, c] == '.': moves.push((r, c))
    for p in moves:
        g[p.0, p.1] = '.'
        g[Row.at(p.0 + 1)!, p.1] = 'W'
    moves.len > 0

for _ in 0..q:
    let kind: 1..=2 = read()
    if kind == 1:
        for i in 0..n:
            let r = Row(n - 1 - i)
            for c in Col:
                if g[r, c] == 'X':
                    g[r, c] = '.'
                    if let below = Row.at(r + 1): g[below, c] = 'X'
        var moving = true
        while moving:
            let a = run_left()
            let b = fall()
            moving = a or b
    else:
        for r in Row: print(g[r, ..])
