// Runnable today/ port of IATI '21 A13 for real Rust `cpl build`.
// Stdin: n, n-1 edges (1-based), init[0..n] (0/1), q, then q queries (ty, u, k).
// Uses top-level recursive fns (working around cpl-emit recursive nested-fn self-capture bug).
// original: corpus/submissions/5052303.cpp

fn ri() -> Int:
    let x: Int = read()
    return x

fn insert_sorted(mut a: List<Int>, v: Int):
    a.push(v)
    var i = a.len - 1
    while i > 0 and a[i - 1] > a[i]:
        let t = a[i - 1]
        a[i - 1] = a[i]
        a[i] = t
        i -= 1

fn upper_bound(a: List<Int>, lim: Int) -> Int:
    var lo = 0
    var hi = a.len
    while lo < hi:
        let mid = (lo + hi) / 2
        if a[mid] <= lim: lo = mid + 1
        else: hi = mid
    return lo

var g_n = 0
var g_q = 0
var g_dlim = 0
var g_inf = 0
var g_adj: List<List<Int>> = []
var g_kids: List<List<Int>> = []
var g_upds_at: List<List<(Int, Int)>> = []
var g_dep: List<Int> = []
var g_tin: List<Int> = []
var g_tout: List<Int> = []
var g_timer = 0
var g_seg: List<Int> = []
var g_bags: List<List<Int>> = []
var g_act: List<List<Int>> = []
var g_st2: List<List<Int>> = []

fn euler(u: Int, p: Int, d: Int) -> Int:
    g_dep[u] = d
    g_tin[u] = g_timer
    g_timer += 1
    for v in g_adj[u]:
        if v != p:
            g_kids[u].push(v)
            let _ = euler(v, u, d + 1)
    g_tout[u] = g_timer - 1
    return 0

fn st_set(o: Int, l: Int, r: Int, idx: Int, v: Int) -> Int:
    if l == r:
        g_seg[o] = v
        return 0
    let mid = (l + r) / 2
    if idx <= mid: let _ = st_set(o * 2, l, mid, idx, v)
    else: let _ = st_set(o * 2 + 1, mid + 1, r, idx, v)
    g_seg[o] = min(g_seg[o * 2], g_seg[o * 2 + 1])
    return 0

fn st_min(o: Int, l: Int, r: Int, ql: Int, qr: Int) -> Int:
    if ql <= l and r <= qr: return g_seg[o]
    let mid = (l + r) / 2
    var ans = g_inf
    if ql <= mid: ans min= st_min(o * 2, l, mid, ql, qr)
    if mid < qr: ans min= st_min(o * 2 + 1, mid + 1, r, ql, qr)
    return ans

fn dfs_act(u: Int) -> Int:
    let d = g_dep[u]
    for pair in g_upds_at[u]:
        let idx = min(g_n + 1, d + pair.0)
        g_bags[idx].push(pair.1)
        var best = g_inf
        for x in g_bags[idx]: best min= x
        let _ = st_set(1, 0, g_dlim - 1, idx, best)
    let first_q = st_min(1, 0, g_dlim - 1, d, g_dlim - 1)
    if first_q < g_q: g_act[first_q].push(u)
    for v in g_kids[u]: let _ = dfs_act(v)
    for pair in g_upds_at[u]:
        let idx = min(g_n + 1, d + pair.0)
        let _ = g_bags[idx].pop()
        var best = g_inf
        for x in g_bags[idx]: best min= x
        let _ = st_set(1, 0, g_dlim - 1, idx, best)
    return 0

fn s2_add(o: Int, l: Int, r: Int, idx: Int, val: Int) -> Int:
    insert_sorted(mut g_st2[o], val)
    if l == r: return 0
    let mid = (l + r) / 2
    if idx <= mid: let _ = s2_add(o * 2, l, mid, idx, val)
    else: let _ = s2_add(o * 2 + 1, mid + 1, r, idx, val)
    return 0

fn s2_count(o: Int, l: Int, r: Int, ql: Int, qr: Int, lim: Int) -> Int:
    if ql <= l and r <= qr: return upper_bound(g_st2[o], lim)
    let mid = (l + r) / 2
    var ans = 0
    if ql <= mid: ans += s2_count(o * 2, l, mid, ql, qr, lim)
    if mid < qr: ans += s2_count(o * 2 + 1, mid + 1, r, ql, qr, lim)
    return ans

g_n = ri()
for _ in 0..g_n:
    g_adj.push([])
    g_kids.push([])
    g_upds_at.push([])
for _ in 0..g_n - 1:
    let u = ri() - 1
    let v = ri() - 1
    g_adj[u].push(v)
    g_adj[v].push(u)
var init_on: List<Int> = []
for _ in 0..g_n: init_on.push(ri())

g_q = ri()
var q_ty: List<Int> = []
var q_u: List<Int> = []
var q_k: List<Int> = []
for qi in 0..g_q:
    let ty = ri()
    let u = ri() - 1
    let k = ri()
    q_ty.push(ty)
    q_u.push(u)
    q_k.push(k)
    if ty == 1: g_upds_at[u].push((k, qi))

g_dep = [0..g_n] _ => 0
g_tin = [0..g_n] _ => 0
g_tout = [0..g_n] _ => 0
let _ = euler(0, -1, 0)

g_dlim = g_n + 2
g_inf = g_q + 5
g_seg = [0..4 * g_dlim + 5] _ => g_inf
for _ in 0..g_dlim: g_bags.push([])
for _ in 0..g_q: g_act.push([])
let _ = dfs_act(0)

var cur_on = [0..g_n] _ => false
for _ in 0..4 * g_n + 5: g_st2.push([])
for u in 0..g_n:
    if init_on[u] == 1:
        cur_on[g_tin[u]] = true
        let _ = s2_add(1, 0, g_n - 1, g_tin[u], g_dep[u])

for qi in 0..g_q:
    if q_ty[qi] == 1:
        for u in g_act[qi]:
            let p = g_tin[u]
            if not cur_on[p]:
                cur_on[p] = true
                let _ = s2_add(1, 0, g_n - 1, p, g_dep[u])
    else:
        let u = q_u[qi]
        print(s2_count(1, 0, g_n - 1, g_tin[u], g_tout[u], g_dep[u] + q_k[qi]))
