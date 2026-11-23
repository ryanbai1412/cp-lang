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
// decreasing).
//
// Suffix block optimality: once a position steps below Q, draining the stack
// peels off disjoint P-intervals. For the top group sharing earliest witness
// `p_thresh = pref_min[cond.max_v]`, every unused element in `p_thresh..max_p`
// up to the first unused element `> cond.max_v` is eligible to witness
// `cond.max_v` (an unused element `> cond.max_v` cannot witness it and blocks
// later positions from crossing left past it). Choosing `min_p` as the largest
// eligible unused element maximizes the block's leading value while still
// satisfying `cond.max_v`; every unused element in `min_p..=hi_bound` with
// value `>= p[min_p]` can then move freely into this block in decreasing order,
// and shrinking `right_end = p_thresh.prev` keeps all blocks pairwise disjoint.

struct Cond(max_p: Pos, max_v: Val)

// Monotone stack of active (max_p, max_v) constraints: both fields decrease.
struct CondStack:
    items: List<Cond> = []

    fn pop_after(mut self, pos: Pos):
        while let last = items.last, pos < last.max_p:
            items.pop()

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

fn solve():
    // Prefix where P and Q already agree needs no changes.
    let first_diff = if let d = Pos.find(i => p[i] != q[i]): d else:
        print(q)
        return
    var used: [Val]Bool = false
    for i in Pos.first!..first_diff: used[q[i]] = true

    fn min_pos(a: ?Pos, b: ?Pos) -> ?Pos:
        var res = a
        res min= b
        res

    // Earliest P-position among unused values <= v.
    let unused_pos: [Val]?Pos = [Val] v => if not used[v]: Some(inv_p[v]) else: None
    var pref_min: [Val]?Pos = None
    var low: ?Pos = None
    for v in Val:
        low = min_pos(low, unused_pos[v])
        pref_min[v] = low

    var conds = CondStack()

    fn forms_inversion(i: Pos, v: Val, had_inv: Bool) -> Bool =
        had_inv or (i > first_diff and v < q[i.prev!] and inv_p[v] < inv_p[q[i.prev!]])

    fn required_cap(i: Pos, v: Val, next_cap: ?Val, had_inv: Bool) -> ?Val:
        if i == first_diff and v == p[i]: return None
        let pos = inv_p[v]
        if not forms_inversion(i, v, had_inv):
            var cap: ?Val = Some(v)
            cap min= next_cap
            cap min= conds.active_cap(pos)
            cap
        elif let ac = conds.active_cap(pos), v > ac:
            ac
        else:
            None

    fn can_place(i: Pos, v: Val, next_cap: ?Val, had_inv: Bool) -> Bool:
        if let cap = required_cap(i, v, next_cap, had_inv):
            pref_min[cap].map(w => inv_p[v] > w) ?? false
        else:
            true

    fn commit_place(i: Pos, v: Val, next_cap: ?Val, had_inv: Bool) -> Bool:
        let pos = inv_p[v]
        var next_inv = had_inv
        if not (i == first_diff and v == p[i]):
            let cap = required_cap(i, v, next_cap, had_inv)
            conds.pop_after(pos)
            if let c = cap: conds.push_if_tighter(pos, c)
            next_inv = forms_inversion(i, v, had_inv)
        q[i] = v
        used[v] = true
        next_inv

    // Largest unused witness in `p_thresh..=hi_p` before any unused value > max_v.
    fn pick_witness(p_thresh: Pos, hi_p: Pos, max_v: Val) -> Pos:
        var best = p_thresh
        for pos in p_thresh..=hi_p if not used[p[pos]]:
            if p[pos] > max_v: break
            if p[pos] >= p[best]: best = pos
        best

    // Match Q as far as possible; on the first failure, step q[i] down and return the suffix start.
    fn match_prefix() -> ?Pos:
        var had_inv = false
        for i in first_diff..=Pos.last!:
            let next_cap = i.next.map(nxt => q[nxt])
            if can_place(i, q[i], next_cap, had_inv):
                had_inv = commit_place(i, q[i], next_cap, had_inv)
            else:
                let v = Val.rev.find(
                    u => u < q[i] and not used[u] and can_place(i, u, None, had_inv)
                )!
                had_inv = commit_place(i, v, None, had_inv)
                return i.next
        None

    if let start = match_prefix():
        var suffix: List<Val> = []
        var right_end: ?Pos = Pos.last
        while let cond = conds.items.pop(), let hi_bound = right_end:
            let p_thresh = pref_min[cond.max_v]!
            var max_p = cond.max_p
            while let top = conds.items.last, pref_min[top.max_v] == Some(p_thresh):
                max_p = top.max_p
                conds.items.pop()
            let min_p = pick_witness(p_thresh, min(max_p.prev!, hi_bound), cond.max_v)
            conds.pop_after(min_p)
            let vals = [
                p[pos] for pos in min_p..=hi_bound if not used[p[pos]] and p[pos] >= p[min_p]
            ]
            for v in vals.sorted().rev:
                used[v] = true
                suffix.push(v)
            right_end = p_thresh.prev
        for v in Val.rev if not used[v]:
            suffix.push(v)
        q[start..] = suffix

    print(q)

solve()
