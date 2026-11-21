// AtCoder WTF 2026 D (dream).
// Find the lexicographically largest permutation <= Q reachable from P.
// Invariant: while matching a prefix of Q, every placed element whose P-position
// precedes a pending upper bound adds a constraint (max_p, max_v) requiring some
// remaining value <= max_v to sit strictly before max_p in P. These constraints
// form a monotone stack (both max_p and max_v strictly decreasing). Once a
// position steps down below Q, the remaining values pack greedily into suffix
// blocks ordered by the active stack thresholds, largest value first.
@!multitest
let n: 1..=2e5 = read()
domain Pos = 1..=n
domain Val = 1..=n
let p: [Pos]Val = read()
var q: [Pos]Val = read()
let inv_p: [Val]Pos = p.inverse()

var used: [Val]Bool = false
let st = Pos.first(i => p[i] != q[i]) ?? n
for i in Pos.first..st: used[q[i]] = true

// Earliest P-position among unused values <= v.
let pref_min: [Val]?Pos = [Val] v => (inv_p[u] for u in Val.first..=v if not used[u]).min()

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

var conds = CondStack()
var seen_inversion = false

fn try_place(i: Pos, next_cap: ?Val) -> Bool:
    let v = q[i]
    let pos = inv_p[v]
    if i == st and v == p[i]:
        used[v] = true
        return true
    let inv = seen_inversion or (i > st and v < q[i.prev!] and pos < inv_p[q[i.prev!]])
    let required_cap: ?Val =
        if not inv:
            let base = if let nc = next_cap: min(v, nc) else: v
            if let ac = conds.active_cap(pos): min(base, ac) else: base
        elif let ac = conds.active_cap(pos), v > ac:
            ac
        else:
            none
    if let cap = required_cap:
        if pref_min[cap].map(w => pos <= w) ?? true: return false
        conds.pop_after(pos)
        conds.push_if_tighter(pos, cap)
    else:
        conds.pop_after(pos)
    seen_inversion = inv
    used[v] = true
    true

// Match Q as far as possible; on the first failure, step q[i] down and break.
var filled = st
for i in st..=Pos.last:
    let next_cap = if let nxt = i.next: q[nxt] else: none
    if try_place(i, next_cap):
        filled = i.ord + 1
    else:
        while true:
            q[i] = Val.last(v => v < q[i] and not used[v])!
            if try_place(i, none): break
        filled = i.ord + 1
        break

// Partition remaining values into suffix blocks by draining the constraint stack.
var block_starts: List<Int> = [filled]
var p_to_block: [Pos]?Int = none
var last_thresh = n
while let mut cond = conds.items.pop():
    let p_thresh = pref_min[cond.max_v]!
    while let top = conds.items.last, pref_min[top.max_v] == some(p_thresh):
        cond.max_p = top.max_p
        conds.items.pop()
    let hi_p = min(cond.max_p.ord, last_thresh)

    var min_p = p_thresh
    for pos in p_thresh.next..hi_p if not used[p[pos]]:
        if p[pos] > cond.max_v: break
        if p[pos] >= p[min_p]: min_p = pos

    conds.pop_after(min_p)
    let block_idx = block_starts.len - 1
    var en = block_starts.last!
    for pos in (min_p.ord..last_thresh).rev if not used[p[Pos(pos + 1)]]:
        if p[Pos(pos + 1)] >= p[min_p]:
            p_to_block[Pos(pos + 1)] = block_idx
            en += 1
    block_starts.push(en)
    last_thresh = p_thresh.ord

for v in Val.rev if not used[v]:
    let b = p_to_block[inv_p[v]] ?? (block_starts.len - 1)
    q[Pos(block_starts[b] + 1)] = v
    block_starts[b] += 1

print(q)
