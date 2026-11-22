// CSES 1193 Labyrinth: shortest A -> B path through '.', printed as URDL moves.
use std.grid.{Dir4, step}
let (n, m): (1..=1000, 1..=1000) = read()
domain Row = 0..n
domain Col = 0..m
type Cell = (Row, Col)
let g: [Row, Col]Char = read()
fn find(ch: Char) -> Cell = [(r, c) for r in Row for c in Col if g[r, c] == ch][0]
fn moves(p: Cell) -> List<(to: Cell, d: Dir4)> =
    [(q, d) for d in Dir4.all() if let q = p.step(d) if g[q] != '#']
let found = bfs(moves, [find('A')])
let arcs = found.arcs(find('B'))
if arcs == none:
    print(false)
    return
print(true)
print(arcs!.len)
print("".join(["URDL"[a.d.k] for a in arcs!]))
