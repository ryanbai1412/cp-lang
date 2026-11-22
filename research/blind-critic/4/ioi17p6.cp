// IOI '17 P6 — Books (runnable proto/cpc port of ioi17p6_draft.cp, post-Drill-3 fix)
// Prototype deltas vs v2.1 draft: no `@grader fn`, no `Marks<V>`, `V.from_idx` returns `V`
// (not `?V`), and explicit cycle hull expansion without `Perm.cycles()`.

let n: 1..=1_000_000 = read()
let s: 0..1_000_000 = read()

domain V = 0..n
var p: [V]Int = read()

var base = 0
var lo_need = s
var hi_need = s
for i in 0..n:
    let vi = V.from_idx(i)!
    let diff = p[vi] - i
    base += if diff < 0: -diff else: diff
    if p[vi] != i:
        if i < lo_need:
            lo_need = i
        if i > hi_need:
            hi_need = i

var cyc_l: [V]Int = 0
var cyc_r: [V]Int = 0
var vis: [V]Bool = false
for i in 0..n:
    let vi = V.from_idx(i)!
    if vis[vi]:
        continue
    var nodes: List<Int> = []
    var cur = i
    var mn = i
    var mx = i
    while not vis[V.from_idx(cur)!]:
        vis[V.from_idx(cur)!] = true
        nodes.push(cur)
        if cur < mn:
            mn = cur
        if cur > mx:
            mx = cur
        cur = p[V.from_idx(cur)!]
    for u in nodes:
        let vu = V.from_idx(u)!
        cyc_l[vu] = mn
        cyc_r[vu] = mx

fn expand(init_l: Int, init_r: Int) -> (Int, Int):
    var l = init_l
    var r = init_r
    var ql = cyc_l[V.from_idx(l)!]
    var qr = cyc_r[V.from_idx(l)!]
    if cyc_l[V.from_idx(r)!] < ql:
        ql = cyc_l[V.from_idx(r)!]
    if cyc_r[V.from_idx(r)!] > qr:
        qr = cyc_r[V.from_idx(r)!]
    while ql < l or qr > r:
        if ql < l:
            l -= 1
            let vl = V.from_idx(l)!
            if cyc_l[vl] < ql:
                ql = cyc_l[vl]
            if cyc_r[vl] > qr:
                qr = cyc_r[vl]
        else:
            r += 1
            let vr = V.from_idx(r)!
            if cyc_l[vr] < ql:
                ql = cyc_l[vr]
            if cyc_r[vr] > qr:
                qr = cyc_r[vr]
    return (l, r)

var cur = expand(s, s)
var l = cur.0
var r = cur.1
var extra = 0

while l > lo_need or r < hi_need:
    var nl = l
    var nr = r
    var cost_l = 0
    while nl > lo_need and nr == r:
        nl -= 1
        cost_l += 2
        let nxt = expand(nl, nr)
        nl = nxt.0
        nr = nxt.1

    if nr > r:
        // Stepping left hit a bridging cycle that also crossed `r`: compare with stepping right.
        var nl2 = l
        var nr2 = r
        var cost_r = 0
        while nr2 < hi_need and nl2 == l:
            nr2 += 1
            cost_r += 2
            let nxt2 = expand(nl2, nr2)
            nl2 = nxt2.0
            nr2 = nxt2.1
        extra += if cost_l < cost_r: cost_l else: cost_r
        l = nl
        r = nr
    else:
        // Left reached `lo_need` without bridging past `r`: must also expand right independently.
        extra += cost_l
        l = nl
        while r < hi_need:
            r += 1
            extra += 2
            let nxt = expand(l, r)
            l = nxt.0
            r = nxt.1

print(base + extra)
