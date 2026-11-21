// AtCoder AGC 079 C: transform 1..=n into permutation a with a reversible
// queue, minimizing reversals (or -1). Reversing time pushes n..=1 and pops a
// back-to-front; every value enters on one of two ends, and the relative order
// of pushes and pops forces a unique parity relation within each connected
// component. Pass 1 walks a backwards keeping unplaced values as a stack of
// same-side runs (splicing runs when a deeper value pops, or popping the bottom
// of the sole run). Pass 2 replays the deque greedily: orient each free
// component to match the active end, pop whenever either end matches, and
// reverse only when forced (a 1-element reversal is free).

enum Side: Back | Front

fn flip(s: Side) -> Side = match s:
    Side.Back => Side.Front
    Side.Front => Side.Back

@!multitest
let n: 1..=2e5 = read()
domain Val = 1..=n
domain Pos = 0..n
let a: [Pos]Val = read()

struct Rel(to: Val, opp: Bool)
struct Run(head: Val, tail: Val, opp_below: Bool)

fn solve() -> ?Int:
    var pred: [Val]?Val = none
    var succ: [Val]?Val = none
    var runs: List<Run> = []
    var rel: [Pos]?Rel = none

    fn swallow(l: Run) -> Run:
        let under = runs.pop()!
        succ[l.tail] = under.head
        pred[under.head] = l.tail
        Run(l.head, under.tail, under.opp_below)

    var unpushed: ?Val = Val.last
    for i in Pos.rev:
        let x = a[i]
        while let v = unpushed, x <= v:
            runs.push(Run(v, v, false))
            unpushed = v.prev
        let top = runs.last!
        if top.head == x:
            if top.tail == top.head:
                runs.pop()
                if top.opp_below: rel[i] = Rel(runs.last!.head, true)
            else:
                let nxt = succ[top.head]!
                runs.last!.head = nxt
                rel[i] = Rel(nxt, false)
        else:
            var l = runs.pop()!
            while let r = runs.last, r.head != x:
                if l.opp_below: return none
                l = swallow(l)
            if runs.is_empty:
                if l.tail != x: return none
                l.tail = pred[l.tail]!
                rel[i] = Rel(l.tail, false)
                runs.push(l)
            else:
                let under = runs.last!
                if under.head == under.tail:
                    rel[i] = Rel(l.head, true)
                    runs.pop()
                    l = if under.opp_below: swallow(l) else: Run(l.head, l.tail, false)
                else:
                    let nxt = succ[under.head]!
                    runs.last!.head = nxt
                    rel[i] = Rel(nxt, false)
                    l.opp_below = true
                runs.push(l)

    var comp: [Val]Rel = Rel(Val.first, false)
    for i in Pos:
        comp[a[i]] = if let r = rel[i]:
            let c = comp[r.to]
            Rel(c.to, c.opp != r.opp)
        else:
            Rel(a[i], false)

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
        let c = comp[v]
        base_side[c.to] ??= if c.opp: flip(active) else: active
        let bs = base_side[c.to]!
        face(if c.opp: flip(bs) else: bs)
        match active:
            Side.Front => q.push_front(v)
            Side.Back => q.push(v)
        while let p = next, let lo = q.first, a[p] == lo or a[p] == q.last!:
            face(if a[p] == lo: Side.Back else: Side.Front)
            match active:
                Side.Front => q.pop()
                Side.Back => q.pop_front()
            next = p.prev
    turns

print(solve() ?? -1)
