@! multitest
let (n, q): (Int, Int) = read()
let b: [1..=n]Int = read()
let ops: [0..q](Int, Int, Int) = read()

domain V = 1..=n + q
domain E = 0..2 * q
var cur: [1..=n]V = [1..=n] i => V(i)
var edges: [E](u: V, v: V) = (u: V(1), v: V(1))
for i in 0..q:
    let (x, y, z) = ops[i]
    let v = V(n + 1 + i)
    edges[E(2 * i)] = (u: cur[x], v: v)
    edges[E(2 * i + 1)] = (u: cur[y], v: v)
    cur[z] = v

let g = Graph.directed(edges)
let rev = g.transpose()
var indeg: [V]Int = [V] u => rev.deg(u)
var ans: [V]?Int = none

let evts = [(val: b[i], st: cur[i]) for i in 1..=n].sorted(by: e => -e.val)
var stk: List<V> = []

fn mark(u: V, val: Int):
    if ans[u] == none:
        ans[u] = val
        stk.push(u)

for (val, st) in evts:
    if let prev = ans[st]:
        if prev == val: continue
        print(-1)
        return
    mark(st, val)
    while let u = stk.pop():
        for a in rev.adj(u): mark(a.to, val)
        for a in g.adj(u):
            indeg[a.to] -= 1
            if indeg[a.to] == 0: mark(a.to, val)

print([ans[V(i)] ?? 0 for i in 1..=n])
