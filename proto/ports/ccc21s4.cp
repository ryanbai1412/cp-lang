// cp: 0.1 c217120+dirty
// CCC 2021 S4 Daily Commute: one-way walkways take a minute each; a subway
// starts at station 1 and reaches the i-th stop of its line after i - 1
// minutes. Each day two stops of the line are swapped; print that day's
// fastest trip from station 1 to station n.
use std.graph.{bfs}

let n: 3..=2e5 = read()
let w: 0..=2e5 = read()
let d: 1..=2e5 = read()
domain Station = 1..=n
domain Stop = 1..=n                         // position on the line; the train is there at minute stop.ord
let walkways: [w](from: Station, to: Station) = read()
var line: [Stop]Station = read() where distinct(line) and line[Stop.first] == Station.first
let swaps: [d](x: Stop, y: Stop) = read() where all(s.x != s.y for s in swaps)

var into: [Station]List<Station> = []
for (from, to) in walkways: into[to].push(from)
let on_foot = bfs(s => into[s], [Station.last])     // minutes to walk to station n, none if impossible

// Ride to a stop, get off and walk. The best stop is kept under the swaps.
fn arrival(i: Stop) -> ?Int: i.ord + on_foot[line[i]]?
var fastest = SegTree<Stop, ?Int>.of(min, arrival)
for (x, y) in swaps:
    line.swap(x, y)
    fastest.refresh(x)
    fastest.refresh(y)
    print(fastest.all!)
