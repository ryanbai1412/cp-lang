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
// peels off disjoint P-intervals `[min_p..=last_thresh]` (shrinking `last_thresh`
// to `p_thresh - 1` after each pop, so blocks never overlap). Any permutation
// inside a block satisfies all remaining constraints, so sorting remaining
// values by `(block_id, -val)` maximizes the suffix lexicographically.

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

fn solve():
    // Prefix where P and Q already agree needs no changes.
    let first_diff = if let d = Pos.first(i => p[i] != q[i]): d else:
        print(q)
        return
    var used: [Val]Bool = false
    for i in Pos.first..first_diff: used[q[i]] = true

    // Earliest P-position among unused values <= v.
    let pref_min: [Val]?Pos = ([Val] v => if not used[v]: some(inv_p[v]) else: none).scan(min)

    var conds = CondStack()
    var seen_inversion = false

    fn forms_inversion(i: Pos, v: Val, had_inv: Bool) -> Bool =
        had_inv or (i > first_diff and v < q[i.prev!] and inv_p[v] < inv_p[q[i.prev!]])

    fn required_cap(i: Pos, v: Val, next_cap: ?Val, had_inv: Bool) -> ?Val:
        if i == first_diff and v == p[i]: return none
        let pos = inv_p[v]
        if not forms_inversion(i, v, had_inv):
            [some(v), next_cap, conds.active_cap(pos)].present().min()
        elif let ac = conds.active_cap(pos), v > ac:
            ac
        else:
            none

    fn can_place(i: Pos, v: Val, next_cap: ?Val, had_inv: Bool) -> Bool:
        if let cap = required_cap(i, v, next_cap, had_inv):
            pref_min[cap].map(w => inv_p[v] > w) ?? false
        else:
            true

    fn commit_place(i: Pos, v: Val, next_cap: ?Val):
        let pos = inv_p[v]
        if not (i == first_diff and v == p[i]):
            let cap = required_cap(i, v, next_cap, seen_inversion)
            conds.pop_after(pos)
            if let c = cap: conds.push_if_tighter(pos, c)
            seen_inversion = forms_inversion(i, v, seen_inversion)
        q[i] = v
        used[v] = true

    // Match Q as far as possible; on the first failure, step q[i] down and stop.
    var suffix_start: ?Pos = none
    for i in first_diff..=Pos.last:
        let next_cap = i.next.map(nxt => q[nxt])
        if can_place(i, q[i], next_cap, seen_inversion):
            commit_place(i, q[i], next_cap)
        else:
            let v = Val.rev.first(u => u < q[i] and not used[u] and can_place(i, u, none, seen_inversion))!
            commit_place(i, v, none)
            suffix_start = i.next
            break

    if let start = suffix_start:
        // Partition remaining values into disjoint suffix blocks by draining the stack.
        var block_of: [Pos]Int = n
        var block_id = 0
        var last_thresh = Pos.last
        while let cond = conds.items.pop():
            let p_thresh = pref_min[cond.max_v]!
            var max_p = cond.max_p
            while let top = conds.items.last, pref_min[top.max_v] == some(p_thresh):
                max_p = top.max_p
                conds.items.pop()
            let hi_p = min(max_p, last_thresh)
            let min_p = (p_thresh..=hi_p).take_while(pos => used[p[pos]] or p[pos] <= cond.max_v).filter(pos => not used[p[pos]]).max(by: pos => p[pos])!
            conds.pop_after(min_p)
            for pos in min_p..=last_thresh if not used[p[pos]] and p[pos] >= p[min_p]:
                block_of[pos] = block_id
            block_id += 1
            last_thresh = p_thresh.prev ?? Pos.first

        let suffix = [v for v in Val if not used[v]].sorted(by: v => (block_of[inv_p[v]], -v.ord))
        for (i, v) in zip(start..=Pos.last, suffix):
            q[i] = v

    print(q)

solve()
