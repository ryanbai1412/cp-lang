use std.grid.{adj4}

let (h, w): (1..=1000, 1..=1000) = read()
domain Row = 1..=h
domain Col = 1..=w
type Cell = (Row, Col)
let g: [Row, Col]Char = read()

var dist: [Row, Col]?Int = None
var queue: List<Cell> = []
for r in Row:
    for c in Col:
        if g[r, c] == 'S':
            dist[r, c] = Some(0)
            queue.push((r, c))

var head = 0
var best: ?Int = None
while head < queue.len:
    let p = queue[head]
    head += 1
    let d = dist[p.0, p.1]!
    if g[p.0, p.1] == 'E':
        best = Some(d)
        break
    for q in p.adj4:
        if g[q.0, q.1] != '#' and dist[q.0, q.1] == None:
            dist[q.0, q.1] = Some(d + 1)
            queue.push(q)
print(best ?? -1)
