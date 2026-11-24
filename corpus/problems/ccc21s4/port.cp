// Source: CCC 2021 S4 (Daily Commute), https://dmoj.ca/problem/ccc21s4
// Problem: N, W, D <= 2e5. One-way walkways take a minute each; a train leaves station 1 along
//   the route S_1..S_N and is at S_i after i - 1 minutes. Each day two route positions (not
//   the first) are swapped; print that day's least minutes from station 1 to station N.
// Solution: the best trip rides to some S_i, gets off and walks: (i - 1) + dist(S_i, N), with
//   dist from one BFS from N over reversed walkways. A min SegTree over route positions
//   rewrites the two swapped ones each day: O(N + W + D log N).
// original: corpus/submissions/4323468.cpp
// Bounds: n: 3..=2e5; w: 0..=2e5; d: 1..=2e5;
//   walkways.from, walkways.to, swaps.x, swaps.y: 1..=n

let n: 3..=2e5 = read()
let w: 0..=2e5 = read()
let d: 1..=2e5 = read()
domain Station = 1..=n
domain Stop = 1..=n  // position on the line; the train is there at minute stop.ord
let walkways: [w](from: Station, to: Station) = read()

// for generated tests: station 1 first, then the others in a rotated order
fn random_line() -> [Stop]Station:
    let turn = rand(0, n - 2)
    [Stop] i => if i.ord == 0: Station.first! else: Station(2 + (i.ord - 1 + turn) % (n - 1))

var line: [Stop]Station = read(gen: random_line)
let swaps: [d](x: Stop, y: Stop) = read()
assert all(s.x != s.y for s in swaps)

var into: [Station]List<Station> = []
for (from, to) in walkways: into[to].push(from)
let on_foot = bfs(s => into[s], from: [Station.last!])  // minutes to walk to station n, none if impossible

// Ride to a stop, get off and walk. The best stop is kept under the swaps.
fn arrival(i: Stop) -> ?Int: i.ord + on_foot[line[i]]?
var fastest = SegTree<Stop, ?Int>(min, [Stop] i => arrival(i))
for (x, y) in swaps:
    line.swap(x, y)
    for i in [x, y]: fastest[i] = arrival(i)
    print(fastest.all!)
