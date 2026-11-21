// AtCoder WTF 2026 D: largest permutation <= Q reachable from P.
// Lexicographic optimality: matching the longest feasible prefix of Q and
// stepping down to the largest feasible value `< q[i]` at the first failure `i`
// is uniquely optimal, because any reachable completion after `i` is smaller
// than any completion that keeps a larger prefix.
//
// Feasibility invariant: placing value `v` at `inv_p[v]` while matching Q
// requires some still-unused value `<= cap` to lie strictly before `inv_p[v]`
// in P whenever `v` sits before a pending tighter upper bound. These active
// constraints `(max_p, max_v)` form a monotone stack (both fields strictly
// decreasing). Once a position steps below Q, the remaining stack constraints
// partition the unused values into disjoint suffix blocks (`domain Block`), and
// placing unused values into their assigned blocks in decreasing value order
// produces the lexicographically largest valid suffix.

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
let pref_min: [Val]?Pos = [Val] v => (if not used[v]: some(inv_p[v]) else: none).scan(min)

var conds = CondStack()
var seen_inversion = false

fn forms_inversion(i: Pos, v: Val) -> Bool =
    seen_inversion or (i > first_diff and v < q[i.prev!] and inv_p[v] < inv_p[q[i.prev!]])

fn required_cap(i: Pos, v: Val, next_cap: ?Val) -> ?Val:
    if i == first_diff and v == p[i]: return none
    let pos = inv_p[v]
    if not forms_inversion(i, v):
        [some(v), next_cap, conds.active_cap(pos)].somes.min()
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
var suffix_start = first_diff
for i in first_diff..=Pos.last:
    let next_cap = i.next.map(nxt => q[nxt])
    if can_place(i, q[i], next_cap):
        commit_place(i, q[i], next_cap)
    else:
        let v = Val.rev.first(u => u < q[i] and not used[u] and can_place(i, u, none))!
        commit_place(i, v, none)
        suffix_start = i.next ?? Pos.last
        break

// Partition remaining values into suffix blocks by draining the constraint stack.
var block_ranges: List<List<Pos>> = []
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
    block_ranges.push([pos for pos in (min_p..=last_thresh).rev if not used[p[pos]] and p[pos] >= p[min_p]])
    last_thresh = p_thresh.prev ?? Pos.first

domain Block = 0..=block_ranges.len
var p_to_block: [Pos]Block = Block.last
var block_cursor: [Block]Pos = suffix_start
var cur_pos = suffix_start
for (idx, members) in block_ranges.items:
    let b = Block(idx)
    block_cursor[b] = cur_pos
    for pos in members: p_to_block[pos] = b
    cur_pos += members.len
block_cursor[Block.last] = cur_pos

for v in Val.rev if not used[v]:
    let b = p_to_block[inv_p[v]]
    q[block_cursor[b]] = v
    block_cursor[b] = block_cursor[b].next ?? Pos.last

print(q)
