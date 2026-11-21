@!multitest
let (n, m): (Int, Int) = read()
domain V = 1..=n
domain E = 0..m
let raw: [E](u: V, v: V) = read()
let g = Graph.undirected(raw)
let arcs = [(u, a.to, a.e) for u in V for a in g.out(u)]
let pref = prefix([V] u => g.deg(u))

let q: Int = read()
let raw_q: [0..q](V, V, Int) = read()
let queries = [
    (ql: pref.prefix(l.l), qr: pref.prefix(r.r), need: k + n - (r - l + 1)) for (l, r, k) in raw_q
]

let blk = 350
fn mo_key(qi: Int) -> (Int, Int):
    let b = queries[qi].ql / blk
    (b, if b % 2 == 0: queries[qi].qr else: -queries[qi].qr)

let order = (0..q).sorted(by: mo_key)

let vmax = bit_ceil(n + 1)
let bsz = 512
let nb = ceil_div(vmax, bsz)
var val_freq: [0..vmax]Int = 0
var b_freq: [0..nb]Int = 0
var val: [V]Int = 0
val_freq[0] = n
b_freq[0] = n
var alive: [E]Int = 0

fn upd(u: V, v: V):
    val_freq[val[u]] -= 1
    b_freq[val[u] / bsz] -= 1
    val[u] ^= v.label
    val_freq[val[u]] += 1
    b_freq[val[u] / bsz] += 1

fn add_edge(ind: Int):
    let (src, dst, e) = arcs[ind]
    alive[e] += 1
    if alive[e] == 2:
        upd(src, dst)
        upd(dst, src)

fn del_edge(ind: Int):
    let (src, dst, e) = arcs[ind]
    if alive[e] == 2:
        upd(src, dst)
        upd(dst, src)
    alive[e] -= 1

var (cur_l, cur_r) = (0, 0)
var ans: [0..q]Int = 0
for qi in order:
    let (ql, qr, need) = queries[qi]
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
    var k = need
    var b = 0
    while k > b_freq[b]:
        k -= b_freq[b]
        b += 1
    var i = b * bsz
    while k > val_freq[i]:
        k -= val_freq[i]
        i += 1
    ans[qi] = i
print(ans, sep: "\n")
