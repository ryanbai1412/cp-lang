# Friction Report: cco17p1

## Vertex numbers are plain Int
- **Where:** `trails: List<(Int, Int)>`, `placed`.
- **What felt wrong:** the output graph's vertices are an index space, but its size is only known
  after the greedy, so there is no domain to name them by.
- **Ideal:** a growable domain (`domain V grows`) whose `new()` hands out the next vertex.

## Dream
```cp
let k: 1..=1e7 = read()
fn pairs(c: Int) -> Int = c * (c - 1) / 2
var g = GraphBuilder()
var left = k
var prev: ?Vertex = None
while left > 0:
    let c = (2..=5000).last_true(c => pairs(c) <= left)!
    left -= pairs(c)
    let ring = g.cycle(c)           // c new vertices, closed into a cycle (c = 2: a double edge)
    if let p = prev: g.edge(p, ring.first)
    prev = Some(ring.last)
print(g)
```

- Tokens: port 199, original 563
- Max input (`gen.py --max`): port 0.00 s, 3456 KB, original 0.00 s, 3584 KB
