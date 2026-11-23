// cpl 0.3.0
// Problem: shortest path in a grid from S to T, 4-neighbour moves, '#' walls.
// Input: h w, then h rows of w chars from ".#ST". Output: steps or -1.
// Guesses: H-PRODUCT-DESTRUCTURE (`let (r, c) = p` on a product position),
// H-ARRAY-SCALAR-FILL, H-MINEQ-COND with an `?Int` right side.

let h = read<1..=1000>()
let w = read<1..=1000>()
domain R = 0..h
domain C = 0..w
domain Cell = (R, C)
let g = read<[R, C]Char>()
let s = g.keys.first(p => g[p] == 'S')!
let t = g.keys.first(p => g[p] == 'T')!

var dist: [Cell]?Int = None
var queue = List<Cell>()
dist[s] = 0
queue.push(s)
var head = 0
while head < queue.len:
    let p = queue[head]
    head += 1
    let (r, c) = p
    for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
        if let nb = Cell.at((Int(r) + dr, Int(c) + dc)), g[nb] != '#':
            if dist[nb] min= dist[p] + 1: queue.push(nb)
print(dist[t] ?? -1)
