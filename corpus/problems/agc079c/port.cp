// AtCoder AGC 079 C: transform 1..=n into permutation a with a reversible
// queue, minimizing reversals (or -1). Reversing time pushes n..=1 and pops a
// back-to-front; every value enters on one of two ends.
//
// Run-stack invariant (Pass 1):
// The stack `runs` partitions the current deque into contiguous chains
// `Run(head, tail, opp_below)` (linked via `pred`/`succ`) ordered newest-to-oldest:
// - All elements in a run were inserted on the same side of the deque (with
//   `head` outermost and `tail` innermost).
// - `opp_below == true` means this run is already committed to the opposite
//   side of the run immediately below it; `opp_below == false` means its side
//   relative to the run below is still free (it may later be merged onto the
//   same side via `swallow`, or committed opposite).
//
// Why each branch preserves the invariant (and why `return none` is forced):
// - Pushing `v` adds a free singleton `Run(v, v, opp_below: false)`.
// - Popping `x == top.head` pops from `top`'s side: the next element of `top`
//   shares `x`'s side (`differ: false`), or if `top` was a singleton with
//   `opp_below`, `x` is opposite to `under.head` (`differ: true`).
// - Popping any other `x` must take `x` from the opposite end of the deque, so
//   every run above `x` must sit on the same side as `top` (merged into `l` via
//   `swallow`). Encountering `l.opp_below` means `l` is already committed
//   opposite to the run below it, so `x` is trapped strictly inside the deque
//   (`return none`). Once `l` reaches `x`:
//   * If `runs` is empty, `x` is exposed iff `x == l.tail` (else trapped);
//     popping it links `x` to the new `l.tail` (`differ: false`).
//   * Otherwise `x == under.head` is opposite `l`: if `under` is a singleton,
//     `x` is opposite `l.head`, `under` vanishes, and if `under.opp_below` held
//     then the run beneath `under` is on the same side as `l` (opposite of
//     opposite), so `l` swallows it; if `under` has >= 2 elements, `x` pops
//     from `under` and `l` stays above it with `opp_below = true`.
//
// Minimal reversals (Pass 2): orienting each free parity component on first
// touch to match `active` avoids any avoidable flip; thereafter every push and
// forced pop fixes the required end, so flipping only when needed (ignoring
// free 1-element flips) achieves the exact minimum reversal count.

enum Side: Back | Front

fn flip(s: Side) -> Side = match s:
    Side.Back => Side.Front
    Side.Front => Side.Back

@!multitest
let n: 1..=2e5 = read()
domain Val = 1..=n
domain Pos = 0..n
let a: [Pos]Val = read()

struct Run:
    var head: Val
    var tail: Val
    var opp_below: Bool

fn solve() -> ?Int:
    var pred: [Val]?Val = None
    var succ: [Val]?Val = None
    var runs: List<Run> = []
    var parity = ParityDsu<Val>.new()

    fn swallow(l: Run) -> Run:
        let under = runs.pop()!
        succ[l.tail] = under.head
        pred[under.head] = l.tail
        Run(l.head, under.tail, under.opp_below)

    fn pop_top_front(x: Val):
        let top = runs.last!
        if top.head == top.tail:
            runs.pop()
            if top.opp_below: parity.unite(x, runs.last!.head, differ: true)
        else:
            let nxt = succ[top.head]!
            runs.last!.head = nxt
            parity.unite(x, nxt, differ: false)

    fn pop_value(x: Val) -> Bool:
        if runs.last!.head == x:
            pop_top_front(x)
            return true
        var l = runs.pop()!
        while let r = runs.last and r.head != x:
            if l.opp_below: return false
            l = swallow(l)
        if runs.is_empty:
            if l.tail != x: return false
            l.tail = pred[l.tail]!
            parity.unite(x, l.tail, differ: false)
            runs.push(l)
            return true
        let under = runs.last!
        if under.head == under.tail:
            parity.unite(x, l.head, differ: true)
            runs.pop()
            l = if under.opp_below: swallow(l) else: Run(l.head, l.tail, false)
        else:
            pop_top_front(x)
            l.opp_below = true
        runs.push(l)
        true

    var unpushed: ?Val = Val.last
    for i in Pos.rev:
        let x = a[i]
        while let v = unpushed and x <= v:
            runs.push(Run(v, v, false))
            unpushed = v.prev
        if not pop_value(x): return None

    var base_side: [Val]?Side = None
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
        while let p = next and let lo = q.first and (a[p] == lo or a[p] == q.last!):
            face(if a[p] == lo: Side.Back else: Side.Front)
            match active:
                Side.Front => q.pop()
                Side.Back => q.pop_front()
            next = p.prev
    turns

print(solve() ?? -1)
