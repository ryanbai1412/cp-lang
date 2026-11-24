# Friction Report: ccc18s3

## Every cell of a grid as a list
- **Where:** `let cells = [(r, c) for r in Row for c in Col]`.
- **What felt wrong:** the cells are the product domain `(Row, Col)`, but a product is not
  iterable directly, so the port builds a list to walk, search and filter.
- **Goal it hurts:** brevity.
- **Ideal code:** `for cam in (Row, Col) if g[cam] == 'C'` and `g.position('S')!`.
- **General fix:** iteration over product domains in row-major order and `position` on grids.
- **Severity:** low

## Dream

```cp
use std.graph.{bfs01}
let (n, m): (4..=100, 4..=100) = read()
domain Row = 1..=n
domain Col = 1..=m
let g: [Row, Col]Char = read()
fn belt(p: (Row, Col)) -> ?Dir4 = Dir4.from("URDL", g[p])
var seen: [Row, Col]Bool = false
for cam in (Row, Col) if g[cam] == 'C':
    for d in Dir4.all():
        for q in cam.ray(d).take_while(q => g[q] != 'W') if belt(q) == None: seen[q] = true
fn moves(p: (Row, Col)) -> List<(to: (Row, Col), w: Int)> =
    [(to: q, w:) for (q, w) in (if let d = belt(p): [(p.step(d)!, 0)] else: [(q, 1) for q in p.adj4])
     if g[q] != 'W' and not seen[q]]
let start = g.position('S')!
let dist = bfs01(moves, from: if seen[start]: [] else: [start])
for p in (Row, Col) if g[p] == '.': print(dist[p] ?? -1)
```

## Measurements

- Tokens: port 357, original 1149
- Max input (`gen.py --max`): port 0.00 s, 3456 KB, original 0.00 s, 3584 KB
- Emitted C++: 28085 bytes
