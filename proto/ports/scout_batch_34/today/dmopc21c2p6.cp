// Runnable port (proto/cp.py lib-run) of dmopc21c2p6.cp
// original: corpus/submissions/3968585.cpp

let (n, q): (Int, Int) = read()
domain Pos = 0..n
domain Val = 0..n
domain Q = 0..q

var pos_of: [Val]Pos = [Val] _ => Pos.first
for p in Pos:
    let raw: Int = read()
    pos_of[Val(raw - 1)] = p

var q1: List<(passes: Int, pos: Int, qid: Q)> = []
var q2: List<(passes: Int, val: Int, qid: Q)> = []
var passes = 0
for qid in Q:
    let kind: Int = read()
    if kind == 1:
        passes += 1
    elif kind == 2:
        let idx: Int = read()
        q1.push((passes: passes, pos: idx - 1, qid: qid))
    else:
        let x: Int = read()
        q2.push((passes: passes, val: x - 1, qid: qid))

var ans: [Q]?Int = [Q] _ => none

// Pass 1: Parallel binary search over values 0..n-1.
var active: [Pos]Int = [Pos] _ => 1
var bit = Fenwick<Pos, Int>(active)

fn set_active(p: Pos, on: Int):
    let d = on - active[p]
    if d != 0:
        active[p] = on
        bit.add(p, d)

fn range_sum(f: Fenwick<Pos, Int>, l: Int, r: Int) -> Int:
    if l > r or r < 0 or l >= n: return 0
    let cl = if l < 0: 0 else: l
    let cr = if r >= n: n - 1 else: r
    f.sum(Pos(cl)..=Pos(cr))

fn solve_s1(lo: Int, hi: Int, qs: List<(passes: Int, pos: Int, qid: Q)>):
    if qs.len == 0: return
    if lo == hi:
        for item in qs:
            ans[item.qid] = lo + 1
        return
    let mid = (lo + hi) / 2
    for v in lo..=mid:
        set_active(pos_of[Val(v)], 0)
    var lq: List<(passes: Int, pos: Int, qid: Q)> = []
    var rq: List<(passes: Int, pos: Int, qid: Q)> = []
    for item in qs:
        let shift = item.pos - item.passes
        let exact = shift >= 0 and active[Pos(shift)] == 1
        let tail = range_sum(bit, max(0, shift), n - 1) >= n - item.pos
        if exact or tail:
            rq.push(item)
        else:
            lq.push(item)
    for v in lo..=mid:
        set_active(pos_of[Val(v)], 1)
    solve_s1(lo, mid, lq)
    for v in lo..=mid:
        set_active(pos_of[Val(v)], 0)
    solve_s1(mid + 1, hi, rq)

solve_s1(0, n - 1, q1)

// Pass 2: Sweep values descending with two Fenwick trees (count and index sum).
var buckets: [Val]List<(passes: Int, qid: Q)> = [Val] _ => []
for item in q2:
    buckets[Val(item.val)].push((passes: item.passes, qid: item.qid))

var cnt_bit = Fenwick<Pos, Int>.new()
var idx_bit = Fenwick<Pos, Int>.new()

fn set_val(p: Pos, sgn: Int):
    cnt_bit.add(p, sgn)
    idx_bit.add(p, sgn * p.to_idx)

fn sum_arith(l: Int, r: Int) -> Int:
    if l > r: 0 else: (l + r) * (r - l + 1) / 2

fn suffix_pos_sum(k: Int) -> Int:
    var lo = 0
    var hi = n
    while lo < hi:
        let mid = (lo + hi) / 2
        if range_sum(cnt_bit, max(0, mid - k), n - 1) >= n - mid:
            hi = mid
        else:
            lo = mid + 1
    let cnt_packed = n - lo
    let cnt_shifted = range_sum(cnt_bit, 0, n - 1) - cnt_packed
    let packed_sum = sum_arith(n - cnt_packed, n - 1)
    let shifted_base = range_sum(idx_bit, 0, lo - k - 1)
    packed_sum + cnt_shifted * k + shifted_base

var vi = n - 1
while vi >= 0:
    let v = Val(vi)
    let p = pos_of[v]
    set_val(p, 1)
    for item in buckets[v]:
        set_val(p, -1)
        let without_v = suffix_pos_sum(item.passes)
        set_val(p, 1)
        let with_v = suffix_pos_sum(item.passes)
        ans[item.qid] = with_v - without_v + 1
    vi -= 1

for qid in Q:
    if let res = ans[qid]:
        print(res)
