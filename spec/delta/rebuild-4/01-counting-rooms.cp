// CSES 1192 Counting Rooms
// in: n m, then n rows of m chars ('.' floor, '#' wall); out: the number of rooms
let (n, m) = read<(Int, Int)>()
domain Row = 0..n
domain Col = 0..m
domain Cell = (Row, Col)
let g = read<[Row, Col]Char>()
let floor_nbrs = p => p.adj4.filter(q => g[q] == '.')
var seen: [Cell]Bool = false
var rooms = 0
for p in Cell:
    if g[p] == '.' and not seen[p]:
        rooms += 1
        for _ in dfs(floor_nbrs, from: [p], seen: mut seen): ()
print(rooms)
