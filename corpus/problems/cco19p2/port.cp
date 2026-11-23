// CCO 2019 P2, Sirtet: rigid shapes of '#' cells (connected by sides) fall
// straight down and come to rest on the floor or on each other. Print the
// final picture.
//
// A shape can fall as far as the floor lets it, and no further than the
// shape directly below any of its cells plus the gap between them: shortest
// distances from the floor over those gap edges.
// Runnable on proto/cpc today.
// original: corpus/submissions/3613676.cpp
let n: 1..=1e6 = read()
let m: 1..=1e6 = read()
domain Row = 0..n
domain Col = 0..m
type Cell = (Row, Col)
let g: [Row, Col]Char = read()

fn adj4(p: Cell) -> List<Cell>:  // v2: p.adj4
    var out: List<Cell> = []
    if let r = p.0 - 1: out.push((r, p.1))
    if let r = p.0 + 1: out.push((r, p.1))
    if let c = p.1 - 1: out.push((p.0, c))
    if let c = p.1 + 1: out.push((p.0, c))
    out

// Label the shapes 1, 2, ...; 0 is the floor.
var shape: [Row, Col]Int = 0
var shapes = 0
for r in Row:
    for c in Col:
        if g[r, c] != '#' or shape[r, c] != 0: continue
        shapes += 1
        shape[r, c] = shapes
        var stack: List<Cell> = [(r, c)]
        while let p = stack.pop():
            for q in adj4(p):
                if g[q] == '#' and shape[q] == 0:
                    shape[q] = shapes
                    stack.push(q)

domain S = 0..=shapes
var gaps: [S]List<(S, Int)> = []
for c in Col:
    var below = S(0)
    var below_row = n
    for i in 0..n:
        let r = Row(n - 1 - i)
        if g[r, c] == '#':
            let s = S(shape[r, c])
            gaps[below].push((s, below_row - (n - 1 - i) - 1))
            below = s
            below_row = n - 1 - i

var drop: [S]?Int = None
var pq = MinHeap<(Int, S)>()
drop[S(0)] = 0
pq.push((0, S(0)))
while let (d, u) = pq.pop():
    if drop[u] != d: continue
    for (v, w) in gaps[u]:
        if drop[v] == None or d + w < drop[v]!:
            drop[v] = d + w
            pq.push((d + w, v))

var out: [Row, Col]Char = '.'
for r in Row:
    for c in Col:
        if g[r, c] == '#':
            out[(r + drop[S(shape[r, c])]!)!, c] = '#'
for r in Row: print(out[r, ..])
