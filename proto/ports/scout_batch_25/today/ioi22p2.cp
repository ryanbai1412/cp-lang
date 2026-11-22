// IOI '22 P2 - Prisoner Challenge (DMOJ: ioi22p2): two bags hold distinct
// coin counts in 1..=n (n <= 5000). Prisoners read the whiteboard (0..=x),
// open one bag, and either overwrite the whiteboard or name the smaller bag
// (-1: A is smaller, -2: B is smaller), using x = 20 (21 states).
//
// A fixed tree over 1..=5588: a node [lo, hi] keeps lo and hi as its own
// endpoints and splits the interior evenly among b children, with
// FANOUT = [3, 3, 3, 3, 3, 2, 2, 1] by depth (sizes 2 -> 4 -> 10 -> 22 ->
// 68 -> 206 -> 620 -> 1862 -> 5588 >= 5000). Whiteboard state (depth t,
// child c) means "both bags lie inside the same depth-(t - 1) node and the
// bag opened last went into child c". Bags alternate by depth (`t % 2`).
// The reader walks its own count down the tree: an endpoint or a different
// child settles the order; the same child descends one level. Total states:
// 1 + sum(FANOUT) = 21, and every play terminates in at most `DEPTH + 1` steps.
// original: corpus/submissions/4783608.cpp

enum Step: Low | High | Child(Int)

let n: 2..=5000 = read()
let fanout: List<Int> = [3, 3, 3, 3, 3, 2, 2, 1]
let depth = fanout.len

// size[t]: values covered by a node at depth t (leaf size[depth] = 2).
var rsize: List<Int> = [2]
for b in fanout.rev: rsize.push(b * rsize.last! + 2)
let size = [rsize[depth - t] for t in 0..=depth]

// first_state[t]: state id of (depth t, child 0) for t >= 1.
var first_state: List<Int> = [0, 1]
for b in fanout: first_state.push(first_state.last! + b)
let states = first_state[depth + 1]

fn path(v: Int) -> List<Step>:
    var out: List<Step> = []
    var lo = 1
    for t in 0..=depth:
        let hi = lo + size[t] - 1
        if v == lo: return out + [Step.Low]
        if v == hi: return out + [Step.High]
        let c = (v - lo - 1) / size[t + 1]
        out.push(Step.Child(c))
        lo += 1 + c * size[t + 1]
    out

// Grader action code when the bag just opened is smaller (`mine == true`) or larger.
fn smaller(bag: Int, mine: Bool) -> Int = if (bag == 0) == mine: -1 else: -2

fn step_action(bag: Int, step: Step, next_base: Int) -> Int = match step:
    Step.Low => smaller(bag, true)
    Step.High => smaller(bag, false)
    Step.Child(next) => next_base + next

print(states)
for t in 0..=depth:
    let branches = if t == 0: 1 else: fanout[t - 1]
    let next_base = if t < depth: first_state[t + 1] else: 0
    let bag = t % 2
    for c in 0..branches:
        var row: List<Int> = [bag]
        for v in 1..=n:
            let p = path(v)
            let act =
                if t == 0: step_action(0, p[0], next_base)
                elif t - 1 >= p.len: 0
                else:
                    match p[t - 1]:
                        Step.Low => smaller(bag, true)
                        Step.High => smaller(bag, false)
                        Step.Child(mine) =>
                            if mine != c: smaller(bag, mine < c)
                            else: step_action(bag, p[t], next_base)
            row.push(act)
        print(row)
