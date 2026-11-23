// Runnable port (proto/cp.py lib-run) of statement-first noi05p2.cp
// original: corpus/submissions/7249409.cpp
// Note: proto/cpc lacks `Pool<Node>` + `Node.Id` and `I32`/`U32` today, so
// parallel List<Int> arrays model the recycled implicit-treap node pool.
let (n, m): (Int, Int) = read()
domain Init = 0..n
let init_a: [Init]Int = read()

let neg_inf = -1000000000000

// Simulated Pool<Node>: index 0 is null sentinel.
var val: List<Int> = [0]
var sz: List<Int> = [0]
var sm: List<Int> = [0]
var pre: List<Int> = [0]
var suf: List<Int> = [0]
var bst: List<Int> = [neg_inf]
var has_same: List<Bool> = [false]
var same_v: List<Int> = [0]
var rev_f: List<Bool> = [false]
var prio: List<Int> = [0]
var lc: List<Int> = [0]
var rc: List<Int> = [0]
var free_ids: List<Int> = []

var rng_state = 2463534242

fn next_prio() -> Int:
    rng_state = (rng_state * 1103515245 + 12345) % 2147483647
    rng_state

fn alloc_node(v: Int) -> Int:
    let p = if v > 0: v else: 0
    let pr = next_prio()
    if free_ids.len > 0:
        let id = free_ids.last!
        free_ids.pop()
        val[id] = v
        sz[id] = 1
        sm[id] = v
        pre[id] = p
        suf[id] = p
        bst[id] = v
        has_same[id] = false
        same_v[id] = 0
        rev_f[id] = false
        prio[id] = pr
        lc[id] = 0
        rc[id] = 0
        return id
    let id = val.len
    val.push(v)
    sz.push(1)
    sm.push(v)
    pre.push(p)
    suf.push(p)
    bst.push(v)
    has_same.push(false)
    same_v.push(0)
    rev_f.push(false)
    prio.push(pr)
    lc.push(0)
    rc.push(0)
    id

fn recycle(root: Int):
    if root == 0: return
    var stk: List<Int> = [root]
    while stk.len > 0:
        let u = stk.last!
        stk.pop()
        if lc[u] != 0: stk.push(lc[u])
        if rc[u] != 0: stk.push(rc[u])
        free_ids.push(u)

fn apply_rev(u: Int):
    if u == 0: return
    rev_f[u] = not rev_f[u]
    swap(mut lc[u], mut rc[u])
    swap(mut pre[u], mut suf[u])

fn apply_same(u: Int, c: Int):
    if u == 0: return
    let s = c * sz[u]
    val[u] = c
    sm[u] = s
    has_same[u] = true
    same_v[u] = c
    rev_f[u] = false
    pre[u] = if s > 0: s else: 0
    suf[u] = if s > 0: s else: 0
    bst[u] = if c >= 0: s else: c

fn push(u: Int):
    if has_same[u]:
        apply_same(lc[u], same_v[u])
        apply_same(rc[u], same_v[u])
        has_same[u] = false
        rev_f[u] = false
    elif rev_f[u]:
        apply_rev(lc[u])
        apply_rev(rc[u])
        rev_f[u] = false

fn pull(u: Int):
    let l = lc[u]
    let r = rc[u]
    let v = val[u]
    sz[u] = sz[l] + 1 + sz[r]
    sm[u] = sm[l] + v + sm[r]
    pre[u] = max(pre[l], sm[l] + v + pre[r])
    suf[u] = max(suf[r], suf[l] + v + sm[r])
    bst[u] = max(max(bst[l], bst[r]), suf[l] + v + pre[r])

fn split(u: Int, k: Int) -> (Int, Int):
    if u == 0: return (0, 0)
    push(u)
    let lsz = sz[lc[u]]
    if k <= lsz:
        let res = split(lc[u], k)
        lc[u] = res.1
        pull(u)
        return (res.0, u)
    let res = split(rc[u], k - lsz - 1)
    rc[u] = res.0
    pull(u)
    (u, res.1)

fn merge(a: Int, b: Int) -> Int:
    if a == 0: return b
    if b == 0: return a
    if prio[a] > prio[b]:
        push(a)
        rc[a] = merge(rc[a], b)
        pull(a)
        return a
    push(b)
    lc[b] = merge(a, lc[b])
    pull(b)
    b

fn build_cartesian(vals: List<Int>) -> Int:
    if vals.len == 0: return 0
    var stk: List<Int> = []
    for i in 0..vals.len:
        let cur = alloc_node(vals[i])
        var last = 0
        while stk.len > 0 and prio[stk.last!] < prio[cur]:
            let top = stk.last!
            stk.pop()
            pull(top)
            last = top
        lc[cur] = last
        if stk.len > 0:
            rc[stk.last!] = cur
        stk.push(cur)
    while stk.len > 1:
        let top = stk.last!
        stk.pop()
        pull(top)
    pull(stk[0])
    stk[0]

var init_list: List<Int> = [init_a[i] for i in Init]
var root = build_cartesian(init_list)

for _ in 0..m:
    let op: Str = read()
    if op == "INSERT":
        let (posi, tot): (Int, Int) = read()
        var vals: List<Int> = []
        for _ in 0..tot:
            let x: Int = read()
            vals.push(x)
        let sp = split(root, posi)
        let mid = build_cartesian(vals)
        root = merge(merge(sp.0, mid), sp.1)
    elif op == "DELETE":
        let (posi, tot): (Int, Int) = read()
        let s1 = split(root, posi - 1)
        let s2 = split(s1.1, tot)
        recycle(s2.0)
        root = merge(s1.0, s2.1)
    elif op == "MAKE-SAME":
        let (posi, tot, c): (Int, Int, Int) = read()
        let s1 = split(root, posi - 1)
        let s2 = split(s1.1, tot)
        apply_same(s2.0, c)
        root = merge(merge(s1.0, s2.0), s2.1)
    elif op == "REVERSE":
        let (posi, tot): (Int, Int) = read()
        let s1 = split(root, posi - 1)
        let s2 = split(s1.1, tot)
        apply_rev(s2.0)
        root = merge(merge(s1.0, s2.0), s2.1)
    elif op == "GET-SUM":
        let (posi, tot): (Int, Int) = read()
        if tot == 0:
            print(0)
        else:
            let s1 = split(root, posi - 1)
            let s2 = split(s1.1, tot)
            print(sm[s2.0])
            root = merge(merge(s1.0, s2.0), s2.1)
    else:
        print(bst[root])
