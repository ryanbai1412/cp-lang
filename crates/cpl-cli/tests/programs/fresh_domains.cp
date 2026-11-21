// Fresh domains end-to-end: runtime-expression domains, existential tuple returns
// (single, renamed, multiple), recursive re-entry save/restore, generic slot reads,
// stdlib index-space creators (`compress`, `components`), and runtime-bounded read fields.
use std.graph.{components}
use std.seq

fn sum_dom<D>(a: [D]Int) -> Int:
    var s = 0
    for i in D: s += a[i]
    s

fn make_seq(k: Int) -> (domain P, [P]Int):
    domain Local = 0..k
    var a: [Local]Int = 0
    for i in Local: a[i] = i.to_idx * 3 + 1
    (Local, a)

fn make_pair(a: Int, b: Int) -> (domain A, domain B, [A]Int, [B]Int):
    domain X = 0..a
    domain Y = 0..b
    let xs = [X] i => i.to_idx + 1
    let ys = [Y] j => (j.to_idx + 1) * 10
    (X, Y, xs, ys)

fn rec_sum(k: Int) -> Int:
    if k <= 0: return 0
    domain Local = 0..k
    var a: [Local]Int = 0
    for i in Local: a[i] = i.to_idx + 1
    let sub = rec_sum(k - 1)
    sum_dom(a) + sub

domain V = read(1..=100)
let m: 0..=100 = read()
let edges: [0..m](u: V, v: V, w: 0..=m) = read()
var by_w: [0..=m]List<(u: V, v: V)> = []
for e in edges:
    by_w[e.w].push((u: e.u, v: e.v))

let g = Graph.undirected([(e.u, e.v) for e in edges])
let (domain K, comp) = components(g)
var comp_sz: [K]Int = 0
for v in V: comp_sz[comp[v]] += 1

domain C = compress([100, 20, 100, 5, 20])
let (domain Q, seq_vals) = make_seq(4)
let (domain A, domain B, xs, ys) = make_pair(3, 2)

print(V.count, by_w[0].len, by_w[m].len)
print(K.count, sum_dom(comp_sz))
print(C.count, sum_dom(seq_vals), sum_dom(xs), sum_dom(ys))
print(rec_sum(4))
