// AtCoder WTF 2026 D: largest permutation <= Q reachable from P.
// While matching Q, placed elements whose P-position precedes a pending bound
// push (max_p, max_v) onto a monotone stack (both decreasing), requiring an
// unused value <= max_v before max_p in P; after stepping down, drain the stack
// into suffix blocks, largest value first.
use std.seq.{inverse}

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

var pref_min: [Val]?Pos = none
var best: ?Pos = none
for v in Val:
    if not used[v]: best = if let b = best: min(b, inv_p[v]) else: inv_p[v]
    pref_min[v] = best

struct Cond(max_p: Pos, max_v: Val)

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

fn try_place(i: Pos, next_cap: ?Val) -> Bool:
    let v = q[i]
    let pos = inv_p[v]
    if i.ord == st and v == p[i]:
        used[v] = true
        return true
    let inv = seen_inversion or (i.ord > st and v < q[(i - 1)!] and pos < inv_p[q[(i - 1)!]])
    var required_cap: ?Val = none
    if not inv:
        let base = if let nc = next_cap: min(v, nc) else: v
        required_cap = if let ac = conds.active_cap(pos): min(base, ac) else: base
    elif let ac = conds.active_cap(pos):
        if v > ac: required_cap = ac
    if let cap = required_cap:
        if let w = pref_min[cap]:
            if pos <= w: return false
        else:
            return false
        conds.pop_after(pos)
        conds.push_if_tighter(pos, cap)
    else:
        conds.pop_after(pos)
    seen_inversion = inv
    used[v] = true
    true

var filled = st
for idx in st..n:
    let i = Pos(idx + 1)
    let next_cap: ?Val = if idx + 1 < n: q[Pos(idx + 2)] else: none
    if try_place(i, next_cap):
        filled = idx + 1
    else:
        while true:
            var cand = (q[i] - 1)!
            while used[cand]: cand = (cand - 1)!
            q[i] = cand
            if try_place(i, none): break
        filled = idx + 1
        break

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
