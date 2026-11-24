// Source: Codeforces 2115B (Gellyfish and Camellia Japonica),
//   https://codeforces.com/contest/2115/problem/B
// limits: 2000 ms, 512 MB
// Problem: an unknown array a of n values becomes b after q operations c[z] := min(c[x], c[y]).
//   Given b and the operations, print any valid a (0 <= a_i <= 1e9), or -1 if none exists.
//   n, q <= 3e5 (sums over tests), multitest.
// Solution: each operation makes a version node fed by the current versions of x and y, a
//   DAG whose sources are the initial slots. Take the final values in decreasing order; the
//   node holding b = v gets v, every unfixed predecessor gets v (its tightest lower bound),
//   and a node whose both inputs are now fixed gets v too (its min). A final node already
//   fixed to another value means -1; unconstrained initial slots get 0. O(n + q) after sort.
// Bounds: n: 1..=300_000; q: 0..=300_000; ops.x, ops.y, ops.z: 1..=n

@!multitest
let (n, q): (1..=300_000, 0..=300_000) = read()
domain Slot = 1..=n
domain Step = 0..q
domain V = 1..=n + q
struct Op(x: Slot, y: Slot, z: Slot)

fn initial(i: Slot) -> V = V.nth(i.ord)
fn version(t: Step) -> V = V.nth(n + t.ord)

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
var val_of: [V]?Int = None

let targets = [(val: b[i], node: cur[i]) for i in Slot].sorted(by: e => desc(e.val))
var work: List<V> = []
fn fix(u: V, v: Int):
    if val_of[u] == None:
        val_of[u] = v
        work.push(u)

for (val, st) in targets:
    if let prev = val_of[st] and prev != val:
        print(-1)
        return
    if val_of[st] == None:
        fix(st, val)
        while let u = work.pop():
            for p in rev[u]: fix(p, val)
            for w in g[u]:
                indeg[w] -= 1
                if indeg[w] == 0: fix(w, val)

print([val_of[initial(i)] ?? 0 for i in Slot])
