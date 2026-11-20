// SAC '22 Code Challenge 1 P3: a side view of a tank. A type 1 event drops
// every 'X' block one row (off the bottom it disappears), then water 'W'
// settles: it runs left along its row while the cell is free and falls into
// a free cell below, until nothing moves. A type 2 event prints the tank.
// Runnable on proto/cpc today.
// original: corpus/submissions/3992599.cpp
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
            while let l = at - 1:
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
            if let below = r + 1:
                if g[r, c] == 'W' and g[below, c] == '.': moves.push((r, c))
    for p in moves:
        g[p.0, p.1] = '.'
        g[(p.0 + 1)!, p.1] = 'W'
    moves.len > 0

for _ in 0..q:
    let kind: 1..=2 = read()
    if kind == 1:
        for i in 0..n:
            let r = Row(n - 1 - i)
            for c in Col:
                if g[r, c] == 'X':
                    g[r, c] = '.'
                    if let below = r + 1: g[below, c] = 'X'
        var moving = true
        while moving:
            let a = run_left()
            let b = fall()
            moving = a or b
    else:
        for r in Row: print(g.row(r))
