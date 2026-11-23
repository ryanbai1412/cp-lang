// CF 1385E Directing Edges: the directed edges alone must be acyclic; then
// orient every undirected edge along one topological order of them.
use std.graph.{directed_of_pairs}

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
for (i, v) in order.items: pos[v] = i

print(yes(true))
for e in es:
    if e.t == 1 or pos[e.x] < pos[e.y]: print(e.x, e.y)
    else: print(e.y, e.x)

@check
fn ok() -> Bool:
    let n: 2..=2e5 = read()
    let m: 1..=2e5 = read()
    domain W = 1..=n
    let es = read<[m](t: 0..=1, x: W, y: W)>()
    let fixed = directed_of_pairs([(e.x, e.y) for e in es if e.t == 1])
    let word: Str = read_output()
    if word == "NO": return topo(fixed) == None
    if word != "YES": return false
    var arcs: List<(W, W)> = []
    for e in es:
        let (u, v) = read_output<(W, W)>()
        if e.t == 1 and (u, v) != (e.x, e.y): return false
        if e.t == 0 and (u, v) != (e.x, e.y) and (u, v) != (e.y, e.x): return false
        arcs.push((u, v))
    topo(directed_of_pairs(arcs)) != None
