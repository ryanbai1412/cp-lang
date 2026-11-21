// AtCoder AGC 079 C: transform 1..=n into permutation a with a reversible
// queue, minimizing reversals (or -1). Reversing time pushes n..=1 and pops a
// back-to-front; every value enters on one of two ends.
//
// Run-stack invariant (Pass 1):
// The stack `runs` partitions the current deque into contiguous chains
// `Run(vals, opp_below)` ordered newest-to-oldest:
// - All elements in `vals` were inserted on the same side of the deque (with
//   `vals.first` outermost and `vals.last` innermost).
// - `opp_below == true` means this run is already committed to the opposite
//   side of the run immediately below it; `opp_below == false` means its side
//   relative to the run below is still free (it may later be merged onto the
//   same side via `swallow`, or committed opposite).
//
// Why each branch preserves the invariant (and why `return none` is forced):
// - Pushing `v` adds a free singleton `Run([v], opp_below: false)`.
// - Popping `x == top.vals.first` pops from `top`'s side: the next element of
//   `top` shares `x`'s side (`differ: false`), or if `top` was a singleton with
//   `opp_below`, `x` is opposite to `under.vals.first` (`differ: true`).
// - Popping any other `x` must take `x` from the opposite end of the deque, so
//   every run above `x` must sit on the same side as `top` (merged into `l` via
//   `swallow`). Encountering `l.opp_below` means `l` is already committed
//   opposite to the run below it, so `x` is trapped strictly inside the deque
//   (`return none`). Once `l` reaches `x`:
//   * If `runs` is empty, `x` is exposed iff `x == l.vals.last` (else trapped);
//     popping it links `x` to the new `l.vals.last` (`differ: false`).
//   * Otherwise `x == under.vals.first` is opposite `l`: if `under` is a
//     singleton, `x` is opposite `l.vals.first`, `under` vanishes, and if
//     `under.opp_below` held then the run beneath `under` is on the same side
//     as `l` (opposite of opposite), so `l` swallows it; if `under` has >= 2
//     elements, `x` pops from `under` and `l` stays above it with
//     `opp_below = true`.
//
// Minimal reversals (Pass 2): orienting each free parity component on first
// touch to match `active` avoids any avoidable flip; thereafter every push and
// forced pop fixes the required end, so flipping only when needed (ignoring
// free 1-element flips) achieves the exact minimum reversal count.

use std.dsu.ParityDsu

enum Side: Back | Front

fn flip(s: Side) -> Side = match s:
    Side.Back => Side.Front
    Side.Front => Side.Back

struct Run(vals: LinkedList<Val>, opp_below: Bool)

@!multitest
let n: 1..=2e5 = read()
domain Val = 1..=n
domain Pos = 0..n
let a: [Pos]Val = read()

fn solve() -> ?Int:
    var runs: List<Run> = []
    var parity = ParityDsu<Val>()
    var unpushed = Val.last

    fn swallow(mut l: Run) -> Run:
        let under = runs.pop()!
        l.vals.splice_back(under.vals)
        Run(l.vals, under.opp_below)

    // Pop the front of the top run `x`, linking `x` to the newly exposed front.
    fn pop_top_front(x: Val):
        var top = runs.pop()!
        if top.vals.len == 1:
            if top.opp_below: parity.unite(x, runs.last!.vals.first, differ: true)
        else:
            top.vals.pop_front()
            parity.unite(x, top.vals.first, differ: false)
            runs.push(top)

    // Pop `x` from whichever end of the deque exposes it, or return false if trapped.
    fn pop_value(x: Val) -> Bool:
        if runs.last!.vals.first == x:
            pop_top_front(x)
            return true
        var l = runs.pop()!
        while let r = runs.last, r.vals.first != x:
            if l.opp_below: return false
            l = swallow(mut l)
        if runs.is_empty:
            if l.vals.last != x: return false
            l.vals.pop_back()
            parity.unite(x, l.vals.last, differ: false)
            runs.push(l)
            return true
        let under = runs.last!
        if under.vals.len == 1:
            parity.unite(x, l.vals.first, differ: true)
            runs.pop()
            l = if under.opp_below: swallow(mut l) else: Run(l.vals, false)
        else:
            pop_top_front(x)
            l.opp_below = true
        runs.push(l)
        true

    for i in Pos.rev:
        let x = a[i]
        while let v = unpushed, x <= v:
            runs.push(Run(LinkedList.of(v), false))
            unpushed = v.prev
        if not pop_value(x): return none

    var base_side: [Val]?Side = none
    var active = Side.Back
    var q = Deque<Val>()
    var next: ?Pos = Pos.last
    var turns = 0

    fn face(want: Side):
        if active != want:
            active = want
            if q.len != 1: turns += 1

    for v in Val.rev:
        let (root, opp) = parity.find(v)
        base_side[root] ??= if opp: flip(active) else: active
        let bs = base_side[root]!
        face(if opp: flip(bs) else: bs)
        match active:
            Side.Front => q.push_front(v)
            Side.Back => q.push(v)
        while let p = next, let lo = q.first, a[p] == lo or a[p] == q.last!:
            let pop_side = if a[p] == lo: Side.Back else: Side.Front
            face(pop_side)
            match active:
                Side.Front => q.pop()
                Side.Back => q.pop_front()
            next = p.prev
    turns

print(solve() ?? -1)
