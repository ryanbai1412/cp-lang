// CF 1385E Directing Edges: the directed edges alone must be acyclic; then
// orient every undirected edge along one topological order of them.
@!multitest
let n: 2..=2e5 = read()
let m: 1..=2e5 = read()
domain V = 1..=n
let es = read<[m](t: 0..=1, x: V, y: V)>()

let g = directed_of_pairs([(e.x, e.y) for e in es if e.t == 1])
let order = topo(g) else:
    print(yes(false))
    return
var pos: [V]Int = 0
for (i, v) in order.enumerate(): pos[v] = i

print(yes(true))
for e in es:
    if e.t == 1 or pos[e.x] < pos[e.y]: print(e.x, e.y)
    else: print(e.y, e.x)
