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

struct Rel(to: Val, opp: Bool)
struct Run(vals: LinkedList<Val>, opp_below: Bool)

@!multitest
let n: 1..=2e5 = read()
domain Val = 1..=n
domain Pos = 0..n
let a: [Pos]Val = read()

fn solve() -> ?Int:
    var runs: List<Run> = []
    var rel: [Pos]?Rel = none
    var unpushed = Val.last

    fn swallow(mut l: Run) -> Run:
        let under = runs.pop()!
        l.vals.splice_back(under.vals)
        Run(l.vals, under.opp_below)

    for i in Pos.rev:
        let x = a[i]
        while let v = unpushed, x <= v:
            runs.push(Run(LinkedList.of(v), false))
            unpushed = v.prev
        let top = runs.last!
        if top.vals.first == x:
            if top.vals.len == 1:
                runs.pop()
                if top.opp_below: rel[i] = Rel(runs.last!.vals.first, true)
            else:
                runs.last!.vals.pop_front()
                rel[i] = Rel(runs.last!.vals.first, false)
        else:
            var l = runs.pop()!
            while let r = runs.last, r.vals.first != x:
                if l.opp_below: return none
                l = swallow(mut l)
            if runs.is_empty:
                if l.vals.last != x: return none
                l.vals.pop_back()
                rel[i] = Rel(l.vals.last, false)
                runs.push(l)
            else:
                let under = runs.last!
                if under.vals.len == 1:
                    rel[i] = Rel(l.vals.first, true)
                    runs.pop()
                    l = if under.opp_below: swallow(mut l) else: Run(l.vals, false)
                else:
                    runs.last!.vals.pop_front()
                    rel[i] = Rel(runs.last!.vals.first, false)
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
            let pop_side = if a[p] == lo: Side.Back else: Side.Front
            face(pop_side)
            match active:
                Side.Front => q.pop()
                Side.Back => q.pop_front()
            next = p.prev
    turns

print(solve() ?? -1)
