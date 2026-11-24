// Source: SAC '22 Code Challenge 1 P3 (That Pool), https://dmoj.ca/problem/sac22cc1p3
// limits: 1000 ms, 256 MB
// Problem: an N x M grid (N, M, Q <= 100) of 'X' heads, 'W' blood and '.' empty. Action 1
//   drops every X one row (overwriting the cell below; off the bottom it vanishes), then blood
//   alternates: every W moves to the leftmost cell it can reach in its row, then every W with
//   an empty cell below drops one row, until nothing moves. Action 2 prints the grid.
// Solution: plain simulation. Drop X rows bottom-up, then repeat until stable: slide each W
//   left through empty cells (row by row, left to right, so each W stops at its leftmost
//   reachable cell), then drop every W with an empty cell below, all against the board as it
//   stood before the drop. Each round moves blood down or left, so an action is O((NM)^2).
// original: corpus/submissions/3992599.cpp
// Bounds: n, m, q: 1..=100; kind: 1..=2
let (n, m): (1..=100, 1..=100) = read()
domain Row = 0..n
domain Col = 0..m
var g: [Row, Col]Char = read()
let q: 1..=100 = read()

fn slide_left() -> Bool:
    var moved = false
    for r in Row:
        for c in Col:
            if g[r, c] != 'W': continue
            var at = c
            while let l = at.offset(-1):
                if g[r, l] != '.': break
                (g[r, l], g[r, at]) = ('W', '.')
                (at, moved) = (l, true)
    moved

fn fall() -> Bool:
    var drops: List<(Row, Col)> = []
    for r in Row:
        if let below = r.offset(1):
            for c in Col:
                if g[r, c] == 'W' and g[below, c] == '.': drops.push((r, c))
    for (r, c) in drops: (g[r, c], g[r.offset(1)!, c]) = ('.', 'W')
    not drops.is_empty

for _ in 0..q:
    let kind: 1..=2 = read()
    if kind == 1:
        for r in Row.rev:
            for c in Col:
                if g[r, c] == 'X':
                    g[r, c] = '.'
                    if let below = r.offset(1): g[below, c] = 'X'
        var moving = true
        while moving:
            let slid = slide_left()
            let fell = fall()
            moving = slid or fell
    else:
        for r in Row: print(g[r, ..])
