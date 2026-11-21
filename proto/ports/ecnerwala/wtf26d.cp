// AtCoder WTF 2026 D: largest permutation <= Q reachable from P.
// While matching a prefix of Q, every placed element whose P-position precedes
// a pending upper bound adds a constraint `(max_p, max_v)` requiring some
// remaining value <= `max_v` to sit strictly before `max_p` in P. These
// constraints form a monotone stack (both `max_p` and `max_v` strictly
// decreasing). Once a position steps down below Q, the remaining values pack
// greedily into suffix blocks ordered by the active stack thresholds, largest
// value first.

use std.seq.inverse

@!multitest
let n: 1..=200_000 = read()
domain Pos = 1..=n
domain Val = 1..=n
let p: [Pos]Val = read()
var q: [Pos]Val = read()
let inv_p = p.inverse()

var used: [Val]Bool = false
var st = 0
for i in Pos:
    if p[i] != q[i]: break
    used[q[i]] = true
    st += 1
if st == n:
    print(q)
    return

// Earliest P-position among unused values <= v.
var pref_min: [Val]?Pos = none
var best_pos: ?Pos = none
for v in Val:
    if not used[v]: best_pos = if let b = best_pos: min(b, inv_p[v]) else: inv_p[v]
    pref_min[v] = best_pos

struct Cond(max_p: Pos, max_v: Val)

// Monotone stack of active (max_p, max_v) constraints: both fields decrease.
struct CondStack:
    items: List<Cond>

    fn pop_after(mut self, pos: Pos):
        while let top = items.last, pos < top.max_p: items.pop()

    fn active_cap(self, pos: Pos) -> ?Val:
        if let top = items.last, pos < top.max_p: top.max_v else: none

    fn push_if_tighter(mut self, pos: Pos, cap: Val):
        if let top = items.last, cap >= top.max_v: return
        items.push(Cond(pos, cap))

var conds = CondStack([])
var seen_inversion = false

fn forms_inversion(i: Pos, v: Val) -> Bool =
    seen_inversion or (i.ord > st and v < q[(i - 1)!] and inv_p[v] < inv_p[q[(i - 1)!]])

fn required_cap(i: Pos, v: Val, next_cap: ?Val) -> ?Val:
    if i.ord == st and v == p[i]: return none
    let pos = inv_p[v]
    if not forms_inversion(i, v):
        let base = if let nc = next_cap: min(v, nc) else: v
        if let ac = conds.active_cap(pos): min(base, ac) else: base
    elif let ac = conds.active_cap(pos):
        if v > ac: ac else: none
    else:
        none

fn can_place(i: Pos, v: Val, next_cap: ?Val) -> Bool:
    let cap = required_cap(i, v, next_cap) ?? return true
    if let w = pref_min[cap]: inv_p[v] > w else: false

fn commit_place(i: Pos, v: Val, next_cap: ?Val):
    let pos = inv_p[v]
    if not (i.ord == st and v == p[i]):
        let cap = required_cap(i, v, next_cap)
        conds.pop_after(pos)
        if let c = cap: conds.push_if_tighter(pos, c)
        seen_inversion = forms_inversion(i, v)
    q[i] = v
    used[v] = true

// Match Q as far as possible; on the first failure, step q[i] down and stop.
var filled = st
for idx in st..n:
    let i = Pos(idx + 1)
    let next_cap: ?Val = if idx + 1 < n: q[Pos(idx + 2)] else: none
    if can_place(i, q[i], next_cap):
        commit_place(i, q[i], next_cap)
        filled = idx + 1
    else:
        var cand = (q[i] - 1)!
        while used[cand] or not can_place(i, cand, none): cand = (cand - 1)!
        commit_place(i, cand, none)
        filled = idx + 1
        break

// Partition remaining values into suffix blocks by draining the constraint stack.
var block_starts: List<Int> = [filled]
var p_to_block: [Pos]?Int = none
var last_thresh = n
while let cond = conds.items.pop():
    let p_thresh = pref_min[cond.max_v]!
    var max_p = cond.max_p.ord
    while let top = conds.items.last, pref_min[top.max_v] == some(p_thresh):
        max_p = top.max_p.ord
        conds.items.pop()
    max_p min= last_thresh

    var min_p = p_thresh
    var min_p_val = p[min_p]
    for idx in p_thresh.ord + 1..max_p:
        let pos = Pos(idx + 1)
        if not used[p[pos]]:
            let v = p[pos]
            if v > cond.max_v: break
            if v >= min_p_val:
                min_p = pos
                min_p_val = v

    conds.pop_after(min_p)
    let block_idx = block_starts.len - 1
    var en = block_starts.last!
    for idx in (min_p.ord..last_thresh).rev:
        let pos = Pos(idx + 1)
        if not used[p[pos]] and p[pos] >= min_p_val:
            p_to_block[pos] = block_idx
            en += 1
    block_starts.push(en)
    last_thresh = p_thresh.ord

for v in Val.rev if not used[v]:
    let b = p_to_block[inv_p[v]] ?? (block_starts.len - 1)
    q[Pos(block_starts[b] + 1)] = v
    block_starts[b] += 1

print(q)
