// Source: IOI 2011 P1 (Tropical Garden), https://dmoj.ca/problem/ioi11p1
// Problem: n <= 150000 fountains and m <= 150000 trails listed from most to least
//   beautiful. A walker takes the most beautiful trail at a fountain, unless it is the one
//   just used and there is another; then the second. For each of q <= 2000 lengths
//   k <= 1e9, count the start fountains from which exactly k trails end at fountain p.
// Solution: the next move depends only on (fountain, arrived by its best trail), so 2n states
//   each with one successor. BFS the reversed map from both states at p to get every start's
//   distance; if the goal state lies on a cycle of length lap, a start at distance d also
//   ends there after d + lap, d + 2 lap, ... Fold counts by lap up to 2n and reduce larger k
//   by whole laps. O(n + m + q).
// The original is a grader function; this port reads `n m p q`, the m
// trails, then the q lengths.
// original: corpus/problems/ioi11p1/orig.cpp (submission 3732440 plus the sample grader)
// Bounds: n: 2..=150000; m: 1..=150000; p, ends.a, ends.b: 0..=n - 1; q: 1..=2000;
//   lengths: 1..=1e9

let (n, m): (2..=150000, 1..=150000) = read()
let p: 0..n = read()
let q: 1..=2000 = read()
domain F = 0..n
domain Trail = 0..m  // most beautiful first
domain State = 0..2 * n  // 2 * fountain + (arrived by its best trail)
domain Query = 0..q
let ends: [Trail](a: F, b: F) = read()
let lengths: [Query]1..=1e9 = read()

var best: [F]?Trail = None
var second: [F]?Trail = None
for t in Trail:
    for f in [ends[t].a, ends[t].b]:
        if best[f] == None: best[f] = t
        elif second[f] == None: second[f] = t

fn state(f: F, by_best: Bool) -> State = State(2 * f.ord + Int(by_best))

fn next(s: State) -> State:
    let f = F(s.ord / 2)
    let t = if s.ord % 2 == 1: second[f] ?? best[f]! else: best[f]!
    let to = if ends[t].a == f: ends[t].b else: ends[t].a
    state(to, best[to]! == t)

var back: [State]List<(to: State)> = []
for s in State: back[next(s)].push((to: s))

domain Len = 0..=2 * n
var ans: [Query]Int = 0
for by_best in Bool:
    let goal = state(F(p), by_best)
    let dist = bfs(s => back[s], [goal])
    // starts[d]: fountains d trails away, summed with those a whole number
    // of laps closer when the goal is on a cycle of length lap.
    var starts: [Len]Int = 0
    for f in F:
        if let d = dist[state(f, false)]: starts[Len(d)] += 1
    if let d = dist[next(goal)]:
        let lap = d + 1
        for e in lap..=2 * n: starts[Len(e)] += starts[Len(e - lap)]
        for i in Query:
            // the largest length <= 2n that is k minus whole laps
            let k = lengths[i]
            ans[i] += starts[Len(if k <= 2 * n: k else: 2 * n - (2 * n - k) % lap)]
    else:
        for i in Query:
            if lengths[i] <= 2 * n: ans[i] += starts[Len(lengths[i])]

for i in Query: print(ans[i])
