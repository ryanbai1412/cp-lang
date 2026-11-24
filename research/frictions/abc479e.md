# abc479e friction

There is no `dream.cp`. The port is the ideal, so the `## Dream` is the port as written.

## Dream
`port.cp` as written: `Rooted(...)`, then `tree.lower(u, v)` and `tree.size(c)` decide which
side each cut adds to. A preorder pass `x[v] = down[v] + (if let p = tree.parent(v): x[p] else: base)`
pushes the offsets down.

## No gaps
Edges are their own domain `E = 1..n`, so `e.label` is the edge number the statement adds.
The rooted-tree queries come from std. There is no dream/port gap to route.

## Measurements
- Tokens: port 195, original 423.
- `--max` (best of 10): port 0.043 s / 21 MB, `orig.cpp` (not judge-accepted, stress-checked)
  0.099 s / 19 MB. Same output.
- Emitted C++: 23,238 bytes.
