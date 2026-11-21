// IOI 2011 Tropical Garden: n fountains and m trails, listed from the most
// beautiful down. A walker always takes the most beautiful trail at a
// fountain, unless that is the trail they just arrived by and there is
// another one; then they take the second most beautiful. For each of q
// lengths k, count the starting fountains from which exactly k trails lead
// to fountain p.
//
// What the walker does next depends on the fountain and on whether they
// arrived by its best trail, so there are 2n states and each has exactly
// one next state. Walk that map backwards from each of the two states at p
// to get the distance of every start, and note that p's state can only be
// revisited by going once around its own cycle.
//
// The original is a grader function; this port reads `n m p q`, the m
// trails, then the q lengths.
// original: corpus/submissions/3732440.cpp

let (n, m, p, q): (Int, Int, Int, Int) = read()
domain F = 0..n
domain Trail = 0..m
domain State = 0..2 * n  // 2 * fountain + (arrived by its best trail)
var ends: List<(a: F, b: F)> = []
var best: [F]?Trail = none
var second: [F]?Trail = none
for t in Trail:
    let (a, b): (Int, Int) = read()
    ends.push((a: F(a), b: F(b)))
    for f in [F(a), F(b)]:
        if best[f] == none: best[f] = t
        elif second[f] == none: second[f] = t

fn state(f: F, by_best: Bool) -> State = State(2 * f.to_idx + Int(by_best))

fn next(s: State) -> State:
    let f = F(s.to_idx / 2)
    let t = if s.to_idx % 2 == 1: second[f] ?? best[f]! else: best[f]!
    let to = if ends[t.to_idx].a == f: ends[t.to_idx].b else: ends[t.to_idx].a
    state(to, best[to]! == t)

var back: [State]List<(to: State)> = []
for s in State: back[next(s)].push((to: s))

domain Len = 0..=2 * n
var ans: List<Int> = List.filled(q, 0)
var lengths: List<Int> = []
for _ in 0..q:
    let k: Int = read()
    lengths.push(k)

for by_best in [false, true]:
    let goal = state(F(p), by_best)
    let dist = bfs(s => back[s], [goal])
    // starts[d]: fountains d trails away, summed with those a whole number
    // of laps closer when the goal is on a cycle.
    var starts: [Len]Int = 0
    for f in F:
        if let d = dist[state(f, false)]: starts[Len(d)] += 1
    let lap = (dist[next(goal)] ?? -1) + 1  // 0 when the goal is not on a cycle
    if lap > 0:
        for d in Len:
            if d.to_idx >= lap: starts[d] += starts[Len(d.to_idx - lap)]
    for i in 0..q:
        let k = lengths[i]
        if k <= 2 * n: ans[i] += starts[Len(k)]
        elif lap > 0:
            let top = 2 * n - (2 * n - k) % lap  // the largest length <= 2n that is k minus whole laps
            ans[i] += starts[Len(top)]

for x in ans: print(x)
