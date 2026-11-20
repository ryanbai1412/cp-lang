// IOI '22 P2, Prisoner Challenge (buildable today-form)
// original: corpus/submissions/4783608.cpp (grader function devise_strategy)
// stdin here: N.  stdout: the number of states, then one row per state.
let N: 2..=5000 = read()

let fanout: List<Int> = [3, 3, 3, 3, 3, 2, 2, 1]
let DEPTH = 8

// size[t] = values covered by a node at depth t (size[8] = 2)
var rsize: List<Int> = [2]
for i in 0..DEPTH:
    rsize.push(fanout[DEPTH - 1 - i] * rsize[i] + 2)
fn size(t: Int) -> Int: rsize[DEPTH - t]

// first[t] = state id of (depth t, child 0), t >= 1
var first: List<Int> = [0, 1]
for t in 0..DEPTH:
    first.push(first[t + 1] + fanout[t])
let states = first[DEPTH + 1]

// path of v: child index per depth, ended by -1 (low endpoint) or -2 (high)
fn path(v: Int) -> List<Int>:
    var out: List<Int> = []
    var lo = 1
    var t = 0
    var done = false
    while not done:
        let hi = lo + size(t) - 1
        if v == lo:
            out.push(-1)
            done = true
        elif v == hi:
            out.push(-2)
            done = true
        else:
            let c = (v - lo - 1) / size(t + 1)
            out.push(c)
            lo += 1 + c * size(t + 1)
            t += 1
    out

// the answer when the bag just opened is the smaller (mine) or larger one
fn smaller(bag: Int, mine: Bool) -> Int:
    if (bag == 0) == mine:
        return -1
    -2

fn verdict(bag: Int, step: Int) -> Int: smaller(bag, step == -1)

for state in 0..states:
    var t = 0
    while first[t + 1] <= state:
        t += 1
    let c = state - first[t]
    let bag = t % 2
    var row: List<Int> = [bag]
    for v in 1..(N + 1):
        let p = path(v)
        var act = 0
        if t == 0:
            if p[0] < 0:
                act = verdict(0, p[0])
            else:
                act = first[1] + p[0]
        elif t - 1 < p.len:
            let mine = p[t - 1]
            if mine < 0:
                act = verdict(bag, mine)
            elif mine != c:
                act = smaller(bag, mine < c)
            elif p[t] < 0:
                act = verdict(bag, p[t])
            else:
                act = first[t + 1] + p[t]
        row.push(act)
    if state == 0:
        print(states)
    print(row)
