// Source: IOI 2008 Teleporters, https://dmoj.ca/problem/ioi08p4
// Problem: a track from 0 to 2,000,001 has n <= 1e6 teleporters with distinct endpoints
//   1 <= w < e <= 2e6. Riding east, reaching an endpoint jumps you to the other one, 1 point
//   per jump. Add up to m <= 1e6 new teleporters (any free endpoints); print the most points.
// Solution: each stretch between consecutive endpoints leads to the stretch after its exit's
//   partner, so the stretches form one path from the start (the ride) plus disjoint cycles. A
//   new teleporter from the ride into a cycle splices it in for its length + 2 points: take the
//   longest cycles first; with none left, each pair of crossing teleporters adds 4 and a last
//   single one adds 1. Sorting the endpoints dominates: O(n log n).
// original: corpus/submissions/5637680.cpp
let n: 1..=1e6 = read()
let m: 1..=1e6 = read()
domain Tele = 1..=n
domain Point = 1..=2 * n  // the endpoints, west to east
type Ends = (w: 1..=2e6, e: 1..=2e6)

// All 2n endpoints are different and w < e: no type says that, so the input
// is built here. Spread 2n spots along the track, shuffle, pair them up.
fn track() -> [Tele]Ends:
    var spot: [Point]Int = 0
    var here = 0
    for p in Point:
        here += rand(1, 1e6 / n)
        spot[p] = here
    for p in Point:
        let o = Point(rand(1, p.label))
        let s = spot[p]
        spot[p] = spot[o]
        spot[o] = s
    [Tele] t => (
        min(spot[Point(2 * t.label - 1)], spot[Point(2 * t.label)]),
        max(spot[Point(2 * t.label - 1)], spot[Point(2 * t.label)]),
    )

let teles: [Tele]Ends = read(gen: track)

var ends: List<(Int, Tele)> = []
for t in Tele:
    ends.push((teles[t].w, t))
    ends.push((teles[t].e, t))
let order = ends.sorted()
var partner: [Point]Point = [Point] p => p
var first_end: [Tele]?Point = None
for p in Point:
    let (_, t) = order[p.label - 1]
    if let o = first_end[t]:
        partner[p] = o
        partner[o] = p
    else:
        first_end[t] = p

// Stretch s of track lies just east of point s (stretch 0 is the start).
// Riding stretch s ends at point s + 1 and continues on the stretch east of
// its partner. So the stretches form one path from 0 off the east end, which
// is the ride, plus loops that are never visited.
var seen: [0..=2 * n]Bool = false
fn ride(from: Int) -> Int:  // jumps until the track ends or the ride repeats
    var s = from
    var jumps = 0
    while s < 2 * n and not seen[s]:
        seen[s] = true
        s = partner[Point(s + 1)].label
        jumps += 1
    jumps

var points = ride(0)
var loops: List<Int> = []
for s in 0..2 * n:
    if not seen[s]: loops.push(ride(s))

// A new teleporter with one end on the ride and one on a loop splices the
// whole loop in: its jumps plus 2. With no loops left, two new teleporters
// that cross each other add 4, and a single one adds 1.
var spare = m
for jumps in loops.sorted(by: x => -x):
    if spare > 0:
        points += jumps + 2
        spare -= 1
print(points + 4 * (spare / 2) + spare % 2)
