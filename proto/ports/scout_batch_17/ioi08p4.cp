// IOI 2008 Teleporters: you ride east along a track from its west end. Each
// teleporter has two endpoints, and reaching one moves you to the other, from
// where you keep riding east: one point per jump. With n teleporters in place,
// add m more (any free endpoints) to score as many points as possible.
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
let order = sorted(ends)
var partner: [Point]Point = [Point] p => p
var first_end: [Tele]?Point = none
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
for jumps in sorted(loops, by: x => -x):
    if spare > 0:
        points += jumps + 2
        spare -= 1
print(points + 4 * (spare / 2) + spare % 2)
