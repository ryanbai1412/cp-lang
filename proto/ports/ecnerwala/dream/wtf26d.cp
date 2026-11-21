// AtCoder WTF 2026 D: largest permutation <= Q reachable from P.
// While matching a prefix of Q, every placed element whose P-position precedes
// a pending upper bound adds a constraint `(max_p, max_v)` requiring some
// remaining value <= `max_v` to sit strictly before `max_p` in P. These
// constraints form a monotone stack (both `max_p` and `max_v` strictly
// decreasing). Once a position steps down below Q, the remaining values pack
// greedily into suffix blocks ordered by the active stack thresholds, largest
// value first.

use std.seq.inverse

struct Cond(max_p: Pos, max_v: Val)

// Monotone stack of active (max_p, max_v) constraints: both fields decrease.
struct CondStack:
    items: List<Cond> = []

    fn pop_after(mut self, pos: Pos):
        items.pop_while(c => pos < c.max_p)

    fn active_cap(self, pos: Pos) -> ?Val =
        items.last.filter(c => pos < c.max_p).map(c => c.max_v)

    fn push_if_tighter(mut self, pos: Pos, cap: Val):
        if items.last.map(c => cap < c.max_v) ?? true:
            items.push(Cond(pos, cap))

@!multitest
let n: 1..=200_000 = read()
domain Pos = 1..=n
domain Val = 1..=n
let p: [Pos]Val = read()
var q: [Pos]Val = read()
let inv_p: [Val]Pos = p.inverse()

// Prefix where P and Q already agree needs no changes.
let first_diff = Pos.first(i => p[i] != q[i]) ?? (print(q); return)
var used: [Val]Bool = false
for i in Pos.first..first_diff: used[q[i]] = true

// Earliest P-position among unused values <= v.
var pref_min: [Val]?Pos = none
var best_pos: ?Pos = none
for v in Val:
    if not used[v]: best_pos = if let b = best_pos: min(b, inv_p[v]) else: inv_p[v]
    pref_min[v] = best_pos

var conds = CondStack()
var seen_inversion = false

fn forms_inversion(i: Pos, v: Val) -> Bool =
    seen_inversion or (i > first_diff and v < q[i.prev!] and inv_p[v] < inv_p[q[i.prev!]])

fn required_cap(i: Pos, v: Val, next_cap: ?Val) -> ?Val:
    if i == first_diff and v == p[i]: return none
    let pos = inv_p[v]
    if not forms_inversion(i, v):
        let base = if let nc = next_cap: min(v, nc) else: v
        if let ac = conds.active_cap(pos): min(base, ac) else: base
    elif let ac = conds.active_cap(pos), v > ac:
        ac
    else:
        none

fn can_place(i: Pos, v: Val, next_cap: ?Val) -> Bool:
    let cap = required_cap(i, v, next_cap) ?? return true
    pref_min[cap].map(w => inv_p[v] > w) ?? false

fn commit_place(i: Pos, v: Val, next_cap: ?Val):
    let pos = inv_p[v]
    if not (i == first_diff and v == p[i]):
        let cap = required_cap(i, v, next_cap)
        conds.pop_after(pos)
        if let c = cap: conds.push_if_tighter(pos, c)
        seen_inversion = forms_inversion(i, v)
    q[i] = v
    used[v] = true

// Match Q as far as possible; on the first failure, step q[i] down and stop.
var filled = first_diff.ord
for i in first_diff..=Pos.last:
    let next_cap = i.next.map(nxt => q[nxt])
    if can_place(i, q[i], next_cap):
        commit_place(i, q[i], next_cap)
        filled = i.ord + 1
    else:
        let v = Val.last(u => u < q[i] and not used[u] and can_place(i, u, none))!
        commit_place(i, v, none)
        filled = i.ord + 1
        break

// Partition remaining values into suffix blocks by draining the constraint stack.
var block_starts: List<Int> = [filled]
var p_to_block: [Pos]?Int = none
var last_thresh = Pos.last
while let mut cond = conds.items.pop():
    let p_thresh = pref_min[cond.max_v]!
    while let top = conds.items.last, pref_min[top.max_v] == some(p_thresh):
        cond.max_p = top.max_p
        conds.items.pop()
    let hi_p = min(cond.max_p, last_thresh)

    var min_p = p_thresh
    for pos in p_thresh.next..=hi_p if not used[p[pos]]:
        if p[pos] > cond.max_v: break
        if p[pos] >= p[min_p]: min_p = pos

    conds.pop_after(min_p)
    let block_idx = block_starts.len - 1
    var en = block_starts.last!
    for pos in (min_p..=last_thresh).rev if not used[p[pos]] and p[pos] >= p[min_p]:
        p_to_block[pos] = block_idx
        en += 1
    block_starts.push(en)
    last_thresh = p_thresh.prev ?? Pos.first

for v in Val.rev if not used[v]:
    let b = p_to_block[inv_p[v]] ?? (block_starts.len - 1)
    q[Pos(block_starts[b] + 1)] = v
    block_starts[b] += 1

print(q)
