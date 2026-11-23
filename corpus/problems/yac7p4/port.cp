// YAC 7 P4 (today/ runnable subset) — original: corpus/submissions/5705324.cpp
// Uses a SegTree over Euler preorder positions to support first_ge/min active tin
// since proto/cpc Set<T> lacks ordered first/first_ge.
use std.math.{bit_len}

let n: 1..=200_000 = read()
let m: 1..=200_000 = read()
domain V = 1..=n

var g: [V]List<V> = []
var edges_u: List<V> = []
var edges_v: List<V> = []
for _ in 0..n - 1:
    let u: V = read()
    let v: V = read()
    g[u].push(v)
    g[v].push(u)
    edges_u.push(u)
    edges_v.push(v)

var queries: [0..m]V = read()

// Iterative Euler tour + binary lifting
let logn = bit_len(n)

var par: [V]?V = None
var depth: [V]Int = 0
var tin: [V]Int = 0
var tout: [V]Int = 0
var ord: [0..n]V = V.first!
var timer = 0

// Recursive DFS to compute tin, tout, par, depth
fn dfs(u: V, p: ?V):
    par[u] = p
    tin[u] = timer
    ord[timer] = u
    timer += 1
    for i in 0..g[u].len:
        let v = g[u][i]
        var is_par = false
        if let pp = p:
            if v == pp:
                is_par = true
        if not is_par:
            depth[v] = depth[u] + 1
            dfs(v, u)
    tout[u] = timer

dfs(V.first!, None)

var lift: [0..logn, V]?V = None
for v in V:
    lift[0, v] = par[v]
for k in 1..logn:
    for v in V:
        if let mid = lift[k - 1, v]:
            lift[k, v] = lift[k - 1, mid]

fn is_anc(u: V, v: V) -> Bool:
    tin[u] <= tin[v] and tout[v] <= tout[u]

fn kth_anc(u: V, steps: Int) -> V:
    var cur = u
    var rem = steps
    var k = 0
    while rem > 0:
        if rem % 2 == 1:
            cur = lift[k, cur]!
        rem = rem / 2
        k += 1
    cur

// Min-SegTree over Euler order 0..n: active slot holds its tin, inactive holds INF
const INF = 1_000_000_000
domain Pre = 0..n
var st = SegTree<Pre, Int>(INF, (a, b) => min(a, b))
var active: [V]Bool = true
var active_cnt = n
for v in V:
    st.set(Pre(tin[v]), tin[v])

fn has_active_sub(u: V) -> Bool:
    st.fold(Pre(tin[u]), Pre(tout[u] - 1)) < INF

fn closest_edge(u: V) -> (V, V):
    let top_tin = st.fold(Pre(0), Pre(n - 1))
    let top = ord[top_tin]
    if not is_anc(top, u):
        return (top, par[top]!)
    var cur = u
    for k in (0..logn).rev:
        if let nu = lift[k, cur]:
            if not has_active_sub(nu):
                cur = nu
    let lca = par[cur]!
    let child = kth_anc(u, depth[u] - depth[lca] - 1)
    (lca, child)

// Active adjacency sets & leaf tracking
var adj: [V]Set<V> = []
for i in 0..edges_u.len:
    let u = edges_u[i]
    let v = edges_v[i]
    adj[u].insert(v)
    adj[v].insert(u)

var leaves: List<V> = []
var in_leaves: [V]Bool = false
fn sync_leaf(u: V):
    if adj[u].len == 1:
        if not in_leaves[u]:
            in_leaves[u] = true
            leaves.push(u)
    else:
        in_leaves[u] = false

for v in V:
    sync_leaf(v)

for qi in 0..m:
    let u = queries[qi]
    if active_cnt == 1:
        print(1)
        continue

    var keep: ?V = None
    var ins_c1: ?V = None
    var ins_c2: ?V = None
    if active[u]:
        keep = u
    else:
        let edge = closest_edge(u)
        keep = edge.0
        ins_c1 = edge.0
        ins_c2 = edge.1

    var cur_leaves: List<V> = []
    var next_leaves: List<V> = []
    for i in 0..leaves.len:
        let lv = leaves[i]
        if in_leaves[lv]:
            var is_keep = false
            if let kv = keep:
                if lv == kv:
                    is_keep = true
            if is_keep:
                next_leaves.push(lv)
            else:
                in_leaves[lv] = false
                cur_leaves.push(lv)
    leaves = next_leaves

    for i in 0..cur_leaves.len:
        let a = cur_leaves[i]
        active[a] = false
        active_cnt -= 1
        st.set(Pre(tin[a]), INF)
        var nbrs: List<V> = []
        for v in adj[a]:
            nbrs.push(v)
        for j in 0..nbrs.len:
            let v = nbrs[j]
            adj[v].remove(a)
            sync_leaf(v)
        adj[a].clear()

    if let c1 = ins_c1:
        let c2 = ins_c2!
        active[c2] = true
        active_cnt += 1
        st.set(Pre(tin[c2]), tin[c2])
        adj[c1].insert(c2)
        adj[c2].insert(c1)
        sync_leaf(c1)
        sync_leaf(c2)

    print(active_cnt)
