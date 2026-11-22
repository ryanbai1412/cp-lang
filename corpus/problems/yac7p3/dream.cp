// YAC 7 P3: in a functional graph, redirect one cycle vertex to the deepest tree vertex hanging
// off that cycle, then number every vertex by its position on its final cycle (cycles numbered
// one after another, tree vertices by depth modulo the cycle length).
domain V = [read Int]
var next = read [V]V
let f = Functional(next)
let deep = V.argmax(by: v => f.tail_len(v))!
let entry = f.cycle_entry(deep)
let u = f.cycle_of(entry).find(c => next[c] == entry)!
next[u] = deep
let g = Functional(next)
var label: [V]Int = 0
var base = 0
for cyc in g.cycles():
    for (i, c) in cyc.items:
        for (v, d) in g.tree_from(c).depths(): label[v] = base + (i + d) % cyc.len
    base += cyc.len
print(u, deep)
print(label.map(l => l + 1))
