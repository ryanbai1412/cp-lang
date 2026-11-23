// Codeforces 2115 B: Gellyfish and Camellia Japonica.
// Each operation `c[z] = min(c[x], c[y])` creates a version node `v` with
// edges `cur[x] -> v` and `cur[y] -> v` in a version DAG (`val(v) = min` of
// its two predecessors). Processing final values `(b[i], cur[i])` in
// decreasing order of `b[i]`:
//   - every unfixed predecessor `p` of `u` must have `val(p) >= val(u)`, so
//     fixing `p = val(u)` assigns its tightest lower bound;
//   - when both predecessors of a node `w` are fixed (`indeg[w] == 0`), the
//     last one fixed equals `val(u)` (the minimum so far), forcing `val(w) = val(u)`.
// Any initial slot left unconstrained after all constraints is set to `0`.

@!multitest
let (n, q): (1..=300_000, 0..=300_000) = read()
domain Slot = 1..=n
domain Step = 0..q
domain V = 1..=n + q
struct Op(x: Slot, y: Slot, z: Slot)

fn initial(i: Slot) -> V = V(i.ord + 1)
fn version(t: Step) -> V = V(n + t.ord + 1)

let b: [Slot]Int = read()
let ops: [Step]Op = read()

var cur: [Slot]V = [Slot] i => initial(i)
var edges: List<(V, V)> = []
for t in Step:
    let op = ops[t]
    let v = version(t)
    edges.push((cur[op.x], v))
    edges.push((cur[op.y], v))
    cur[op.z] = v

let g = Graph.directed(edges)
let rev = Graph.directed([(v, u) for (u, v) in edges])
var indeg: [V]Int = [V] u => rev[u].len
var val_of: [V]?Int = none

let targets = [(val: b[i], node: cur[i]) for i in Slot].sorted(by: e => e.val, desc: true)
var work: List<V> = []
fn fix(u: V, v: Int):
    if val_of[u] == none:
        val_of[u] = v
        work.push(u)

for (val, st) in targets:
    if let prev = val_of[st] and prev != val:
        print(-1)
        return
    if val_of[st] == none:
        fix(st, val)
        while let u = work.pop():
            for p in rev[u]: fix(p, val)
            for w in g[u]:
                indeg[w] -= 1
                if indeg[w] == 0: fix(w, val)

print([val_of[initial(i)] ?? 0 for i in Slot])
