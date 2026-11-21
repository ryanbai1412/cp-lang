// CCO '24 P5 — Parity-Alternating Subarray Partition DP (today form for proto/cpc)
// original: corpus/submissions/6435483.cpp
let MOD = 1_000_003
let n: 1..=500_000 = read()

domain I = 0..n
domain P = 0..=n
var arr: [I]Int = 0
for i in 0..n:
    let x: Int = read()
    arr[I(i)] = x - 1

var sz = 1
while sz < n + 1: sz *= 2

// Two mirrored tag-permanent lazy segtrees (parity 0 and parity 1), packed into
// one flat array with base offset `cb * 2 * sz` since proto/cpc lacks struct methods.
let STRIDE = 2 * sz
var seg_mn: List<Int> = []
var seg_ways: List<Int> = []
var seg_lazy: List<Int> = []
for _ in 0..2 * STRIDE:
    seg_mn.push(0)
    seg_ways.push(0)
    seg_lazy.push(0)

fn pull_node(base: Int, o: Int):
    let lc = base + o * 2
    let rc = lc + 1
    let idx = base + o
    let ml = seg_mn[lc]
    let mr = seg_mn[rc]
    if ml < mr:
        seg_mn[idx] = ml + seg_lazy[idx]
        seg_ways[idx] = seg_ways[lc]
    elif mr < ml:
        seg_mn[idx] = mr + seg_lazy[idx]
        seg_ways[idx] = seg_ways[rc]
    else:
        seg_mn[idx] = ml + seg_lazy[idx]
        var s = seg_ways[lc] + seg_ways[rc]
        if s >= MOD: s -= MOD
        seg_ways[idx] = s

fn upd_ways(base: Int, o: Int, l: Int, r: Int, pos: Int, val: Int):
    if l == r:
        seg_ways[base + o] = val
        return
    let mid = (l + r) / 2
    if pos <= mid: upd_ways(base, o * 2, l, mid, pos, val)
    else: upd_ways(base, o * 2 + 1, mid + 1, r, pos, val)
    pull_node(base, o)

fn upd_bad(base: Int, o: Int, l: Int, r: Int, ql: Int, qr: Int, d: Int):
    if ql <= l and r <= qr:
        seg_lazy[base + o] += d
        seg_mn[base + o] += d
        return
    let mid = (l + r) / 2
    if ql <= mid: upd_bad(base, o * 2, l, mid, ql, qr, d)
    if mid < qr: upd_bad(base, o * 2 + 1, mid + 1, r, ql, qr, d)
    pull_node(base, o)

fn range_bad(base: Int, ql: Int, qr: Int, d: Int):
    if ql <= qr: upd_bad(base, 1, 0, n, ql, qr, d)

fn query_zero(base: Int) -> Int:
    if seg_mn[base + 1] > 0: return 0
    seg_ways[base + 1]

var dp: [P]Int = 0
dp[P(n)] = 1

var nxt0: [I]Int = -1
var nxt1: [I]Int = -1
var g_nxt: [I]Int = -1
var dup0: [I]Int = -1
var dup1: [I]Int = -1

var i = n - 1
while i >= 0:
    let x = arr[I(i)]
    for cb in 0..2:
        let base = cb * STRIDE
        upd_ways(base, 1, 0, n, i + 1, dp[P(i + 1)])

        let unq = if cb == 0: nxt1[I(x)] else: nxt0[I(x)]
        if unq != -1: range_bad(base, unq + 1, n, 1)

        if i % 2 != cb:
            if g_nxt[I(x)] != -1: range_bad(base, g_nxt[I(x)] + 1, n, 1)
        else:
            let dn = if cb == 0: nxt0[I(x)] else: nxt1[I(x)]
            var nset = n
            if dn != -1:
                let prev_end = if cb == 0: dup0[I(dn)] else: dup1[I(dn)]
                range_bad(base, dn + 1, prev_end, -1)
                nset = dn
            if cb == 0: dup0[I(i)] = nset
            else: dup1[I(i)] = nset
            range_bad(base, i + 1, nset, 1)

        var cur = dp[P(i)] + query_zero(base)
        if cur >= MOD: cur -= MOD
        dp[P(i)] = cur

    if i % 2 == 0: nxt0[I(x)] = i
    else: nxt1[I(x)] = i
    g_nxt[I(x)] = i
    i -= 1

print(dp[P(0)])
