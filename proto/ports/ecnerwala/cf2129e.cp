@!multitest
let (n, m): (Int, Int) = read()
var edges: List<(Int, Int, Int)> = []
var deg: [0..n]Int = 0
for e in 0..m:
    let (u0, v0): (Int, Int) = read()
    let u = u0 - 1
    let v = v0 - 1
    edges.push((u, v, e))
    edges.push((v, u, e))
    deg[u] += 1
    deg[v] += 1
edges.sort()

var vmax = 1
while vmax <= n: vmax *= 2

var deg_pref: [0..=n]Int = 0
for i in 0..n: deg_pref[i + 1] = deg_pref[i] + deg[i]

let q: Int = read()
var queries: List<(Int, Int, Int)> = []
var qpos: List<(Int, Int)> = []
for _ in 0..q:
    let (l0, r, k): (Int, Int, Int) = read()
    let l = l0 - 1
    queries.push((l, r, k))
    qpos.push((deg_pref[l], deg_pref[r]))

let blk = 350
var order: List<(Int, Int, Int)> = [(qpos[qi].0 / blk, if ((qpos[qi].0 / blk) & 1) == 0: qpos[qi].1 else: -qpos[qi].1, qi) for qi in 0..q]
order.sort()

let bsz = 512
let nb = (vmax - 1) / bsz + 1
var val_freq: [0..vmax]Int = 0
var b_freq: [0..nb]Int = 0
var val: [0..n]Int = 0
val_freq[0] = n
b_freq[0] = n
var alive: [0..m]Int = 0

fn upd(u: Int, v: Int):
    val_freq[val[u]] -= 1
    b_freq[val[u] / bsz] -= 1
    val[u] ^= v + 1
    val_freq[val[u]] += 1
    b_freq[val[u] / bsz] += 1

fn add_edge(ind: Int):
    let (src, dst, e) = edges[ind]
    alive[e] += 1
    if alive[e] == 2:
        upd(src, dst)
        upd(dst, src)

fn del_edge(ind: Int):
    let (src, dst, e) = edges[ind]
    if alive[e] == 2:
        upd(src, dst)
        upd(dst, src)
    alive[e] -= 1

var cur_l = 0
var cur_r = 0
var ans: [0..q]Int = 0
for (_, _, qi) in order:
    let (ql, qr) = qpos[qi]
    while cur_l > ql:
        cur_l -= 1
        add_edge(cur_l)
    while cur_r < qr:
        add_edge(cur_r)
        cur_r += 1
    while cur_l < ql:
        del_edge(cur_l)
        cur_l += 1
    while cur_r > qr:
        cur_r -= 1
        del_edge(cur_r)
    let (l, r, k0) = queries[qi]
    var k = k0 + n - (r - l)
    var b = 0
    while k > b_freq[b]:
        k -= b_freq[b]
        b += 1
    var i = b * bsz
    while k > val_freq[i]:
        k -= val_freq[i]
        i += 1
    ans[qi] = i
for x in ans: print(x)
