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
// original: corpus/submissions/3732440.cpp

let (n, m, p, q): (Int, Int, Int, Int) = read()
domain F = 0..n
domain Trail = 0..m
domain State = 0..2 * n  // 2 * fountain + (arrived by its best trail)
var ends: List<(a: F, b: F)> = []
var best: [F]?Trail = None
var second: [F]?Trail = None
for t in Trail:
    let (a, b): (Int, Int) = read()
    ends.push((a: F(a), b: F(b)))
    for f in [F(a), F(b)]:
        if best[f] == None: best[f] = t
        elif second[f] == None: second[f] = t

fn state(f: F, by_best: Bool) -> State = State(2 * f.ord + Int(by_best))

fn next(s: State) -> State:
    let f = F(s.ord / 2)
    let t = if s.ord % 2 == 1: second[f] ?? best[f]! else: best[f]!
    let to = if ends[t.ord].a == f: ends[t.ord].b else: ends[t.ord].a
    state(to, best[to]! == t)

var back: [State]List<(to: State)> = []
for s in State: back[next(s)].push((to: s))

domain Len = 0..=2 * n
var ans: List<Int> = List.filled(q, 0)
var lengths: List<Int> = []
for _ in 0..q:
    let k: Int = read()
    lengths.push(k)

for by_best in Bool:
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
            if d.ord >= lap: starts[d] += starts[Len(d.ord - lap)]
    for i in 0..q:
        let k = lengths[i]
        if k <= 2 * n: ans[i] += starts[Len(k)]
        elif lap > 0:
            let top = 2 * n - (2 * n - k) % lap  // the largest length <= 2n that is k minus whole laps
            ans[i] += starts[Len(top)]

for x in ans: print(x)
