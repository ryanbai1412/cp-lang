@!multitest
let (n, m, q): (Int, Int, Int) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()
let g = Graph.directed(edges)
let rev = g.transpose()

var val: [V]Int = 0
var freq: [V, 0..4]Int = 0
var is_red: [V]Bool = false

fn get_val(v: V) -> Int:
    if is_red[v]: return 3
    if g.deg(v) == 0: return 0
    if freq[v, 2] > 0 or (freq[v, 0] > 0 and freq[v, 3] > 0): return 1
    if freq[v, 3] > 0: return 3
    if freq[v, 0] > 0: return 0
    2

for v in peel(rev):
    val[v] = get_val(v)
    for a in rev.out(v): freq[a.to, val[v]] += 1

fn propagate(v: V):
    let old_v = val[v]
    let new_v = get_val(v)
    if new_v == old_v: return
    val[v] = new_v
    for a in rev.out(v):
        freq[a.to, old_v] -= 1
        freq[a.to, new_v] += 1
        propagate(a.to)

for _ in 0..q:
    let (op, v): (Int, V) = read()
    if op == 1:
        is_red[v] = true
        propagate(v)
    else:
        print(if val[v] <= 1: "YES" else: "NO")
