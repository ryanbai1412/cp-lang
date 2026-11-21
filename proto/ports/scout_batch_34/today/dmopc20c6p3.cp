// Runnable port (proto/cp.py lib-run) of dmopc20c6p3.cp
// original: corpus/submissions/3917634.cpp

let (n, m): (Int, Int) = read()
domain V = 1..=n
domain E = 0..m

let want_raw: [V]Int = read()
let want: [V]Bool = [V] u => want_raw[u] == 1
let edges: [E](u: V, v: V) = read()

let g = Graph<V, E>.directed(edges)
let rg = g.transpose()

var out_deg: [V]Int = [V] u => g.deg(u)
var rem: [V]Int = [V] u => out_deg[u]
var queued: [V]Bool = [V] _ => false
var state: [V]?Bool = [V] _ => none
var q: List<V> = []

fn enqueue(u: V, force: Bool):
    if not queued[u] and (force or rem[u] == 0):
        queued[u] = true
        q.push(u)

for u in V:
    enqueue(u, false)

while q.len > 0:
    let u = q.pop()
    var res = true
    if out_deg[u] == 0:
        res = true
    elif want[u]:
        res = false
        for arc in g.out(u):
            if let sv = state[arc.to]:
                if sv: res = true
    else:
        res = true
        for arc in g.out(u):
            if not state[arc.to]!:
                res = false
    state[u] = res
    for arc in rg.out(u):
        let p = arc.to
        rem[p] -= 1
        enqueue(p, want[p] and res)

var out = ""
for u in V:
    let bit = if (state[u] ?? false): "1" else: "0"
    if u.to_idx > 0: out += " "
    out += bit
print(out)
