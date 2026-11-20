// SAC '22 Code Challenge 1 P3: a side view of a tank. A type 1 event drops
// every 'X' block one row (off the bottom it disappears), then water 'W'
// settles: it runs left along its row while the cell is free and falls into
// a free cell below, until nothing moves. A type 2 event prints the tank.
// original: corpus/submissions/3992599.cpp
let (n, m): (1..=50, 1..=50) = read()
domain Row = 0..n
domain Col = 0..m
type Cell = (Row, Col)
var g: [Row, Col]Char = read()
let q: 1..=50 = read()

fn run_left() -> Bool:
    var changed = false
    for p in Cell:
        if g[p] != 'W': continue
        var at = p
        while let l = at.step(Dir4.W) and g[l] == '.':
            (g[l], g[at]) = ('W', '.')
            at = l
            changed = true
    changed

// Every drop falls on the board as it stood before this pass.
fn fall() -> Bool:
    let drops = [p for p in Cell if g[p] == 'W' and p.step(Dir4.S).map(b => g[b]) == '.']
    for p in drops: (g[p], g[p.step(Dir4.S)!]) = ('.', 'W')
    not drops.is_empty

for _ in 0..q:
    let kind: 1..=2 = read()
    if kind == 1:
        for r in Row.rev:
            for c in Col:
                if g[r, c] == 'X':
                    g[r, c] = '.'
                    if let below = r + 1: g[below, c] = 'X'
        while run_left() | fall(): pass       // FRICTION S2: both must run; `or` would short-circuit
    else:
        for r in Row: print(g.row(r))
