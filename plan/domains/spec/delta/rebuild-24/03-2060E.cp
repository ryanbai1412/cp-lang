// CF 2060E Graph Composition. Drop every F edge that crosses G's components, then add
// one edge per G edge that still joins two F components.
@!multitest
let n: 1..=2e5 = read()
let m1, m2: 0..=2e5 = read()
domain V = 1..=n
let f = read<[m1](u: V, v: V)>()
let g = read<[m2](u: V, v: V)>()

var in_g = Dsu<V>()
for e in g: in_g.unite(e.u, e.v)
var in_f = Dsu<V>()
var ops = 0
for e in f:
    if in_g.same(e.u, e.v): in_f.unite(e.u, e.v)
    else: ops += 1
for e in g:
    if in_f.unite(e.u, e.v): ops += 1
print(ops)
