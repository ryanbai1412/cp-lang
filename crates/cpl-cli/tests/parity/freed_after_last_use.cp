// `arcs` is last read by `Graph.directed`, `w` in a loop's body by index,
// and `xs` by a view `head` that later lines still read: the first two are
// freed after their last statement, `xs` stays whole for `head`.
@!multitest
let n: 1..=10 = read()
domain V = 0..n
let arcs = read<[n](V, V)>()
let w: [V]Int = read()
let xs = read<[n]Int>()
let head = xs.take(2)
let g = Graph.directed(arcs)
var s = 0
for v in V:
    s += w[v] * g[v].len
for v in V:
    print(s + g[v].len + head.sum())
