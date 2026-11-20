// IOI '22 P2 - Prisoner Challenge (DMOJ: ioi22p2)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Two bags hold different numbers of coins in 1..=N (N <= 5000). Prisoners
// enter one at a time, read the whiteboard (a number in 0..=x), look into one
// bag, and either overwrite the whiteboard or name the bag with fewer coins.
// Give a strategy s[state][0] = bag to open, s[state][j] = action on seeing j
// coins (-1: A is smaller, -2: B is smaller, otherwise the new state), with
// x as small as possible (x = 20 scores full marks).
//
// A fixed tree over 1..=5588: a node [lo, hi] keeps lo and hi as its own
// "endpoints" and splits the interior evenly among b children, with
// b = 3,3,3,3,3,2,2,1 by depth (leaf size 2 -> 4 -> 10 -> 22 -> 68 -> ... -> 5588).
// The whiteboard says (depth t, child c): "both bags are inside the same
// depth-(t-1) node, and the bag opened last went into child c". Bags alternate
// by depth. The reader walks its own count down the tree; an endpoint or a
// different child settles the order, the same child goes one level deeper.
// 1 + (3*5 + 2*2 + 1) = 21 states.
const FANOUT = [3, 3, 3, 3, 3, 2, 2, 1]
const DEPTH = FANOUT.len

enum Step = Low | High | Child(Int)     // where a value sits in a node

@grader
fn devise_strategy(N: 1..=5000) -> List<List<Int>>:
    // size[t] = number of values covered by a node at depth t
    var size = [2] * (DEPTH + 1)
    for t in (0..DEPTH).rev: size[t] = FANOUT[t] * size[t + 1] + 2

    // path[v][t] = what v does in its depth-t node (stops at the first Low / High)
    fn path(v: Int) -> List<Step>:
        var (lo, out) = (1, [])
        for t in 0..=DEPTH:
            let hi = lo + size[t] - 1
            if v == lo: return out + [Low]
            if v == hi: return out + [High]
            let c = (v - lo - 1) / size[t + 1]
            out.push(Child(c))
            lo += 1 + c * size[t + 1]
        unreachable

    let first_state = prefix([1] + FANOUT)        // state id of (depth t, child 0), t >= 1
    let smaller = (bag: Int, mine: Bool) => if (bag == 0) == mine: -1 else: -2

    var s: List<List<Int>> = [[0] * (N + 1) for _ in 0..first_state[DEPTH + 1]]
    for v in 1..=N:
        let p = path(v)
        // root: open A
        s[0][v] = match p[0]:
            Low => -1
            High => -2
            Child(c) => first_state[1] + c
        for t in 1..=DEPTH:
            let bag = t % 2
            for c in 0..FANOUT[t - 1]:
                let state = first_state[t] + c
                s[state][0] = bag
                // v's step inside the shared depth-(t-1) node, then inside child c
                s[state][v] =
                    if t - 1 >= p.len: 0                      // v cannot be here
                    else: match p[t - 1]:
                        Low => smaller(bag, true)
                        High => smaller(bag, false)
                        Child(mine) =>
                            if mine != c: smaller(bag, mine < c)
                            else: match p[t]:
                                Low => smaller(bag, true)
                                High => smaller(bag, false)
                                Child(next) => first_state[t + 1] + next
    s

@checker
fn sound(N: Int, s: List<List<Int>>) -> Bool
    where N <= 60:
    all(play(s, a, b) == (if a < b: -1 else: -2) for a in 1..=N for b in 1..=N if a != b)

fn play(s: List<List<Int>>, a: Int, b: Int) -> Int:
    var state = 0
    for _ in 0..500:
        let act = s[state][if s[state][0] == 0: a else: b]
        if act < 0: return act
        state = act
    0
