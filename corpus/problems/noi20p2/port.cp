// Runnable port (proto/cp.py lib-run) of statement-first noi20p2.cp
// original: corpus/submissions/5655455.cpp
// Note: proto/cpc lacks `Pool<Node>` + `Node.Id` today, so parallel List<Int>
// arrays model the node pool (exact same O(N log N) implicit segtree merge).

let mod = 998244353

let n: Int = read()
domain V = 1..=n
domain E = 0..(n - 1)
let edges: [E](u: V, v: V) = read()
let m: Int = read()
domain P = 0..m
let paths: [P](u: V, v: V) = read()

let g = Graph<V, E>.undirected(edges)
let tree = Rooted(u => g.out(u), V.first)

var need: [V]Int = [V] _ => 0
for p in P:
    let du = tree.depth(paths[p].u) + 1
    let v = paths[p].v
    if du > need[v]: need[v] = du

// Simulated Pool<Node>: index 0 is null sentinel.
var lc: List<Int> = [0]
var rc: List<Int> = [0]
var nd_sum: List<Int> = [0]
var nd_mul: List<Int> = [1]

fn alloc_node() -> Int:
    let id = lc.len
    lc.push(0)
    rc.push(0)
    nd_sum.push(0)
    nd_mul.push(1)
    id

fn apply_mul(u: Int, k: Int):
    if u == 0: return
    nd_sum[u] = nd_sum[u] * k % mod
    nd_mul[u] = nd_mul[u] * k % mod

fn push(u: Int):
    if nd_mul[u] != 1:
        apply_mul(lc[u], nd_mul[u])
        apply_mul(rc[u], nd_mul[u])
        nd_mul[u] = 1

fn set_point(u_in: Int, lo: Int, hi: Int, pos: Int, val: Int) -> Int:
    var u = u_in
    if u == 0: u = alloc_node()
    if lo == hi:
        nd_sum[u] = val % mod
        return u
    push(u)
    let mid = (lo + hi) / 2
    if pos <= mid: lc[u] = set_point(lc[u], lo, mid, pos, val)
    else: rc[u] = set_point(rc[u], mid + 1, hi, pos, val)
    nd_sum[u] = (nd_sum[lc[u]] + nd_sum[rc[u]]) % mod
    u

fn query_sum(u: Int, lo: Int, hi: Int, ql: Int, qr: Int) -> Int:
    if u == 0 or qr < lo or hi < ql: return 0
    if ql <= lo and hi <= qr: return nd_sum[u]
    push(u)
    let mid = (lo + hi) / 2
    (query_sum(lc[u], lo, mid, ql, qr) + query_sum(rc[u], mid + 1, hi, ql, qr)) % mod

fn merge_trees(a: Int, b: Int, lo: Int, hi: Int, pre_a: Int, pre_b: Int) -> Int:
    if a == 0 and b == 0: return 0
    if a == 0:
        apply_mul(b, pre_a)
        return b
    if b == 0:
        apply_mul(a, pre_b)
        return a
    if lo == hi:
        let sa = nd_sum[a]
        let sb = nd_sum[b]
        nd_sum[a] = (sa * (pre_b + sb) + sb * pre_a) % mod
        return a
    push(a)
    push(b)
    let mid = (lo + hi) / 2
    let al_sum = nd_sum[lc[a]]
    let bl_sum = nd_sum[lc[b]]
    lc[a] = merge_trees(lc[a], lc[b], lo, mid, pre_a, pre_b)
    rc[a] = merge_trees(rc[a], rc[b], mid + 1, hi, (pre_a + al_sum) % mod, (pre_b + bl_sum) % mod)
    nd_sum[a] = (nd_sum[lc[a]] + nd_sum[rc[a]]) % mod
    a

var roots: [V]Int = [V] _ => 0
for u in tree.at.rev:
    let d_u = tree.depth(u) + 1
    var rt = set_point(0, 0, n, need[u], 1)
    for c in tree.children(u):
        let c_rt = roots[c]
        let zero_case = query_sum(c_rt, 0, n, 0, d_u)
        rt = merge_trees(rt, c_rt, 0, n, 0, zero_case)
    roots[u] = rt

print(query_sum(roots[V.first!], 0, n, 0, 0))
