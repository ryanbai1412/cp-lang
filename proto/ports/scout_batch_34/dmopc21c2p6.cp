// DMOPC '21 Contest 2 P6 — Bubble Sort Queries (CP Lang v2 spec)
// original: corpus/submissions/3968585.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Two offline passes over a permutation undergoing bubble-sort passes:
//   - Type 2 (value at position `p` after `t` passes): parallel binary search
//     over values using a single Fenwick tree of indicators `[val > mid]`.
//   - Type 3 (position of value `v` after `t` passes): sweep values descending;
//     final_pos(v) = sum_final_pos(>= v) - sum_final_pos(> v) via binary search
//     over two Fenwick trees (count and initial-index sum).

let (n, q): (1..=3e5, 1..=3e5) = read()
domain Pos = 0..n
domain Val = 1..=n
domain Q = 0..q

let a: [Pos]Val = read()
var pos_of: [Val]Pos = Pos.first
for (p, v) in a.items:
    pos_of[v] = p

type PosQuery = (passes: Int, pos: Pos, qid: Q)
type ValQuery = (passes: Int, val: Val, qid: Q)

var q_at_pos: List<PosQuery> = []
var q_pos_of: List<ValQuery> = []
var passes = 0
for qid in Q:
    let kind: 1..=3 = read()
    if kind == 1:
        passes += 1
    elif kind == 2:
        let p: 1..=n = read()
        q_at_pos.push((passes: passes, pos: Pos.from_idx(p - 1)!, qid: qid))
    else:
        let v: Val = read()
        q_pos_of.push((passes: passes, val: v, qid: qid))

var ans: [Q]?Int = none

// Pass 1: Parallel binary search for type-2 queries.
// FRICTION UD1: Fenwick has `.add(i, d)` but no `.set(i, v)` / point-read, so
// toggling a 0/1 indicator during parallel binary search requires a parallel
// `active: [Pos]Int` array (or a `BitFenwick` / `Fenwick.set` helper).
var active: [Pos]Int = 1
var bit = Fenwick<Pos, Int>(active)

fn set_active(p: Pos, on: Int):
    let delta = on - active[p]
    if delta != 0:
        active[p] = on
        bit.add(p, delta)

fn solve_at_pos(lo: Val, hi: Val, qs: List<PosQuery>):
    if qs.is_empty: return
    if lo == hi:
        for item in qs: ans[item.qid] = lo.to_idx + 1
        return
    let mid = (lo..=hi).mid!
    for v in lo..=mid: set_active(pos_of[v], 0)
    var left_qs: List<PosQuery> = []
    var right_qs: List<PosQuery> = []
    for item in qs:
        let shift = item.pos.to_idx - item.passes
        let start = Pos.from_idx(max(0, shift))!
        let hits_tail = bit.sum(start..) >= n - item.pos.to_idx
        let hits_exact = if let s = Pos.from_idx(shift): active[s] == 1 else: false
        if hits_exact or hits_tail:
            right_qs.push(item)
        else:
            left_qs.push(item)
    for v in lo..=mid: set_active(pos_of[v], 1)
    solve_at_pos(lo, mid, left_qs)
    for v in lo..=mid: set_active(pos_of[v], 0)
    if let next_lo = mid + 1:
        solve_at_pos(next_lo, hi, right_qs)

solve_at_pos(Val.first, Val.last, q_at_pos)

// Pass 2: Sweep values descending for type-3 queries.
// FRICTION UD2: `sort(mut q_pos_of, by: q => -q.val.to_idx)` needs `.to_idx`
// because `Desc<Val>` works inside tuples (`by: q => desc(q.val)`), which is
// non-obvious compared to `-q.val.to_idx`.
sort(mut q_pos_of, by: item => desc(item.val))

var cnt_bit = Fenwick<Pos, Int>(+)
var idx_bit = Fenwick<Pos, Int>(+)

fn set_val(p: Pos, on: Bool):
    let sgn = if on: 1 else: -1
    cnt_bit.add(p, sgn)
    idx_bit.add(p, sgn * p.to_idx)

fn sum_arith(l: Int, r: Int) -> Int:
    if l > r: 0 else: (l + r) * (r - l + 1) / 2

fn suffix_pos_sum(k: Int) -> Int:
    // FRICTION UD7: Single-expression lambdas (`x => expr`) cannot bind a local
    // `let`, so a two-line predicate for `bsearch_first` needs a named helper.
    fn fits_suffix(mid: Int) -> Bool:
        let tail = if let s = Pos.from_idx(max(0, mid - k)): cnt_bit.sum(s..) else: 0
        tail >= n - mid

    // Smallest l in 0..=n whose suffix [max(0, l - k)..n) holds >= n - l active items.
    let l = bsearch_first(0..=n, fits_suffix)!
    let cnt_packed = n - l
    let cnt_shifted = cnt_bit.sum(Pos.first..) - cnt_packed
    let packed_sum = sum_arith(n - cnt_packed, n - 1)
    let shifted_base = if let end = Pos.from_idx(l - k - 1): idx_bit.sum(..=end) else: 0
    packed_sum + cnt_shifted * k + shifted_base

var qi = 0
for v in (Val.first..=Val.last).rev:
    let p = pos_of[v]
    set_val(p, true)
    while qi < q_pos_of.len and q_pos_of[qi].val == v:
        let item = q_pos_of[qi]
        set_val(p, false)
        let without_v = suffix_pos_sum(item.passes)
        set_val(p, true)
        let with_v = suffix_pos_sum(item.passes)
        ans[item.qid] = with_v - without_v + 1
        qi += 1

for qid in Q:
    if let res = ans[qid]:
        print(res)
