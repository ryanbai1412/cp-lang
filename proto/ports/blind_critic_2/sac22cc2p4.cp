// SAC '22 Code Challenge 2 P4: walk from the top-left to the bottom-right cell of a grid,
// stepping to any of the four neighbours, through as few 'C' cells as possible.
// Dijkstra over the cells: the best route can double back up or left, so no DP order works.
let n: 1..=1000 = read()
let m: 1..=1000 = read()
domain Row = 1..=n
domain Col = 1..=m

fn random_grid() -> [Row, Col]Char:
    [Row, Col] (r, c) => if rand(0, 2) == 0: '.' else: 'C'

let grid: [Row, Col]Char = read(gen: random_grid)

fn cost(r: Row, c: Col) -> Int: if grid[r, c] == 'C': 1 else: 0

var dist: [Row, Col]?Int = none
var pq = MinHeap<(Int, Row, Col)>()
pq.push((cost(Row.first, Col.first), Row.first, Col.first))
while let (d, r, c) = pq.pop():
    if dist[r, c] != none: continue
    dist[r, c] = d
    for (dr, dc) in [(0, 1), (1, 0), (0, -1), (-1, 0)]:
        if let r2 = r + dr and let c2 = c + dc:
            if dist[r2, c2] == none: pq.push((d + cost(r2, c2), r2, c2))
print(dist[Row.last, Col.last]!)

