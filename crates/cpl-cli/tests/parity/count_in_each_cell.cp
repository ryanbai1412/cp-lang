// `xs.count(v)` walks `xs` once per cell of the builder: the list must
// stay whole until the last cell, not be freed after the first.
let n: 1..=100 = read()
domain V = 1..=n
let xs: [n]V = read()
let deg = [V] v => xs.count(v)
for v in V: print(deg[v])
