// YAC 7 P3: in a functional graph, redirect one cycle vertex to the deepest tree vertex hanging
// off that cycle, then number every vertex by its position on its final cycle (cycles numbered
// one after another, tree vertices by depth modulo the cycle length).
let n: 1..=1_000_000 = read()
domain V = 1..=n
var next: [V]V = read()
let f = Functional.new(next)
// hang the cycle vertex that enters the deepest tree onto that tree's deepest vertex
let deep = V.filter(v => not f.on_cycle(v)).max(by: v => (f.depth(v), v)) ?? V.first!
let u = V.find(v => f.on_cycle(v) and next[v] == f.root_of(deep))!
next[u] = deep
let g = Functional.new(next)
// label each cycle consecutively; a tree vertex takes its root's slot shifted by its depth
var spot: [V](base: Int, i: Int) = (base: 0, i: 0)
var base = 0
for cyc in g.cycles():
    for (i, c) in cyc.items: spot[c] = (base:, i: (cyc.len - i) % cyc.len)
    base += cyc.len
fn label(v: V) -> Int:
    let s = spot[g.root_of(v)]
    s.base + (s.i + g.depth(v)) % g.cycle_len(v) + 1
print(u, deep)
print(V.map(label))
