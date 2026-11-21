let (n, q): (1..=200_000, 1..=200_000) = read()
let RT = 1000
domain Small = 0..=RT

var cur = SortedBag<Int>()
var small_gaps: [Small]SortedBag<Int> = [Small] _ => SortedBag<Int>()
var last_small: [Small]Int = 0
var cnt_small: [Small]Int = 0
var tot = 0

fn upd_gap(lo: Int, hi: Int, ins: Bool):
    let l = hi - lo
    if l > RT: return
    let d = Small(l)
    if ins:
        let _ = small_gaps[d].add(lo)
    else:
        let _ = small_gaps[d].remove(lo)
    last_small[d] = small_gaps[d].last ?? 0

fn insert_barrel(v: Int):
    tot += v
    if v <= RT: cnt_small[Small(v)] += 1
    let had = cur.count(v)
    let _ = cur.add(v)
    if had >= 1:
        upd_gap(v, v, true)
    else:
        let prv = cur.last_lt(v)
        let nxt = cur.first_gt(v)
        if prv != none and nxt != none: upd_gap(prv!, nxt!, false)
        if prv != none: upd_gap(prv!, v, true)
        if nxt != none: upd_gap(v, nxt!, true)

fn remove_barrel(v: Int):
    let had = cur.count(v)
    let _ = cur.remove(v)
    if had >= 2:
        upd_gap(v, v, false)
    else:
        let prv = cur.last_lt(v)
        let nxt = cur.first_gt(v)
        if prv != none: upd_gap(prv!, v, false)
        if nxt != none: upd_gap(v, nxt!, false)
        if prv != none and nxt != none: upd_gap(prv!, nxt!, true)
    if v <= RT: cnt_small[Small(v)] -= 1
    tot -= v

fn has_bad_k() -> Bool:
    var cur_val = cur.last!
    if cur.count(cur_val) == 1:
        var cur_tot = tot - cur_val
        var cur_gap = cur_val
        while let nv = cur.last_lt(cur_val):
            cur_tot -= nv
            cur_gap = min(cur_gap, min(cur_val - nv, nv))
            if cur_tot < cur_gap: return true
            if cur_gap <= RT: break
            if cur.count(nv) >= 2: break
            cur_val = nv

    var last_gap = 0
    var cnt_big = cur.len
    var tot_small = 0
    for k in 1..=RT:
        let d = Small(k - 1)
        last_gap max= last_small[d]
        cnt_big -= cnt_small[d]
        tot_small += (k - 1) * cnt_small[d]
        if cnt_big < 2: break
        if last_gap < k and tot_small < k: return true
    false

repeat n:
    let a: 1..=1_000_000 = read()
    insert_barrel(a)
print(if has_bad_k(): "No" else: "Yes")

repeat q:
    let op: Char = read()
    let a: 1..=1_000_000 = read()
    if op == '+': insert_barrel(a)
    else: remove_barrel(a)
    print(if has_bad_k(): "No" else: "Yes")
