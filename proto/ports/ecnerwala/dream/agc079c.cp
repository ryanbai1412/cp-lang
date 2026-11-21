// AtCoder AGC 079 C: transform 1..=n into permutation a with a reversible
// queue, minimizing reversals (or -1). Reversing time pushes n..=1 and pops a
// back-to-front; every value enters on one of two ends.
//
// Invariants:
// - Unique parity relation per component: whenever two values stay in the deque
//   simultaneously, the order they leave fixes their relative insertion side:
//   popping the front of a multi-element run forces the next element to share
//   its side (same parity), while popping a deeper run's head or a singleton
//   marked `opp_below` forces opposite sides (opposite parity).
// - Minimal reversals: in Pass 2, orienting each free component on first touch
//   to match the active end avoids any avoidable flip; thereafter every push
//   and forced pop fixes the required active end, so flipping only when needed
//   (ignoring free 1-element flips) achieves the exact minimum reversal count.

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

    for i in Pos.rev:
        let x = a[i]
        while let v = unpushed, x <= v:
            runs.push(Run(LinkedList.of(v), false))
            unpushed = v.prev
        if runs.last!.vals.first == x:
            pop_top_front(x)
        else:
            var l = runs.pop()!
            while let r = runs.last, r.vals.first != x:
                if l.opp_below: return none
                l = swallow(mut l)
            if runs.is_empty:
                if l.vals.last != x: return none
                l.vals.pop_back()
                parity.unite(x, l.vals.last, differ: false)
                runs.push(l)
            else:
                let under = runs.last!
                if under.vals.len == 1:
                    parity.unite(x, l.vals.first, differ: true)
                    runs.pop()
                    l = if under.opp_below: swallow(mut l) else: Run(l.vals, false)
                else:
                    pop_top_front(x)
                    l.opp_below = true
                runs.push(l)

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
