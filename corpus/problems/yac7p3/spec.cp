// yac7p3: re-point one cycle vertex at the deepest tail vertex so that tail joins the cycle, then
// label every vertex by its position on its cycle, shifted by its depth below it.
let n = read<1..=1_000_000>()
domain V = 1..=n
var par = read<[V]V>()
let f = Functional.new(par)
let deep = V.filter(v => not f.on_cycle(v)).max(by: v => (f.depth(v), v)) ?? V.first!
let moved = V.find(v => f.on_cycle(v) and par[v] == f.root_of(deep))!
par[moved] = deep
let g = Functional.new(par)
var idx: [V]Int = 0
var base: [V]Int = 0
var offset = 0
for cycle in g.cycles():
    for (i, v) in cycle.items:
        idx[v] = i
        base[v] = offset
    offset += cycle.len
fn label(v: V) -> Int:
    let r = g.root_of(v)
    base[r] + (idx[r] + g.depth(v)) % g.cycle_len(v) + 1
print(moved, deep)
print(V.map(label))
