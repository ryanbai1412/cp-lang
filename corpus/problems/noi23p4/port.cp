// Source: NOI 2023 P4 (Trade), https://dmoj.ca/problem/noi23p4
// limits: 1500 ms, 512 MB
// Problem: 2^n - 1 cities (2 <= n <= 18) in a heap-shaped tree: city i >= 2 has a one-way road to
//   i/2 of length a_i, plus m <= 2^n directed roads u -> v with v in u's subtree (lengths <= 1e9).
//   Print the sum of dist(x, y) over all ordered pairs (0 if unreachable) mod 998244353.
// Solution: leaving x's subtree means first stepping up to its parent, so outside it
//   dist(x, y) = a_x + dist(parent, y). A DFS from the root keeps d = dist(x, .) - dw(x),
//   dw being x's road sum to the root: the parent's values already give every path via the
//   parent, then a Dijkstra seeded at x, restricted to x's subtree, improves the inside. Add the
//   reached values plus offsets, recurse, and restore the subtree's values on return.
//   O((2^n + m) n log 2^n).
// original: corpus/submissions/5723063.cpp
// copies: 1

const MODV = 998244353
const INF = 1_000_000_000_000_000_000

fn ri() -> Int:
    let x: Int = read()
    return x

fn norm(x: Int) -> Int:
    var r = x % MODV
    if r < 0: r += MODV
    return r

var g_n = 0
var g_tw: List<Int> = []
var g_adj: List<List<(Int, Int)>> = []
var g_d: List<Int> = []
var g_pos = 0
var g_sum = 0
var g_ans = 0
var g_hd: List<Int> = []
var g_hu: List<Int> = []
var g_sbuf: List<Int> = []

fn set_d(u: Int, v: Int):
    if g_d[u] < INF:
        g_pos -= 1
        g_sum = norm(g_sum - g_d[u])
    g_d[u] = v
    if v < INF:
        g_pos += 1
        g_sum = norm(g_sum + v)

fn h_push(d: Int, u: Int):
    g_hd.push(d)
    g_hu.push(u)
    var i = g_hd.len - 1
    while i > 0:
        let p = (i - 1) / 2
        if g_hd[p] <= g_hd[i]: break
        let td = g_hd[p]
        let tu = g_hu[p]
        g_hd[p] = g_hd[i]
        g_hu[p] = g_hu[i]
        g_hd[i] = td
        g_hu[i] = tu
        i = p

fn h_pop() -> (Int, Int):
    let res = (g_hd[0], g_hu[0])
    let ld = g_hd.pop()!
    let lu = g_hu.pop()!
    if g_hd.len > 0:
        g_hd[0] = ld
        g_hu[0] = lu
        var i = 0
        let n = g_hd.len
        while true:
            var s = i
            let l = 2 * i + 1
            let r = 2 * i + 2
            if l < n and g_hd[l] < g_hd[s]: s = l
            if r < n and g_hd[r] < g_hd[s]: s = r
            if s == i: break
            let td = g_hd[i]
            let tu = g_hu[i]
            g_hd[i] = g_hd[s]
            g_hu[i] = g_hu[s]
            g_hd[s] = td
            g_hu[s] = tu
            i = s
    return res

fn dijkstra(src: Int, init_w: Int):
    g_hd = []
    g_hu = []
    if init_w < g_d[src]:
        set_d(src, init_w)
        h_push(init_w, src)
    while g_hd.len > 0:
        let top = h_pop()
        let cur = top.0
        let u = top.1
        if cur > g_d[u]: continue
        let p = u / 2
        if p >= src and cur + g_tw[u] < g_d[p]:
            set_d(p, cur + g_tw[u])
            h_push(cur + g_tw[u], p)
        for e in g_adj[u]:
            let v = e.0
            let w = e.1
            if v >= src and cur + w < g_d[v]:
                set_d(v, cur + w)
                h_push(cur + w, v)

fn save_sub(u: Int) -> Int:
    if u >= g_n: return 0
    g_sbuf.push(g_d[u])
    let _ = save_sub(u * 2)
    let _ = save_sub(u * 2 + 1)
    return 0

fn load_sub(u: Int, vals: List<Int>, idx: Int) -> Int:
    if u >= g_n: return idx
    set_d(u, vals[idx])
    var next = idx + 1
    next = load_sub(u * 2, vals, next)
    next = load_sub(u * 2 + 1, vals, next)
    return next

fn walk(u: Int, dw: Int) -> Int:
    g_sbuf = []
    let _ = save_sub(u)
    let saved = g_sbuf
    dijkstra(u, -dw)
    g_ans = norm(g_ans + g_sum + norm(dw) * g_pos)
    if u * 2 < g_n:
        let _ = walk(u * 2, dw + g_tw[u * 2])
        let _ = walk(u * 2 + 1, dw + g_tw[u * 2 + 1])
    let _ = load_sub(u, saved, 0)
    return 0

let logn = ri()
let m = ri()
g_n = 1 << logn
g_tw = [0..g_n] _ => 0
for i in 2..g_n: g_tw[i] = ri()
for _ in 0..g_n: g_adj.push([])
for _ in 0..m:
    let u = ri()
    let v = ri()
    let w = ri()
    g_adj[u].push((v, w))
g_d = [0..g_n] _ => INF
let _ = walk(1, 0)
print(g_ans)
