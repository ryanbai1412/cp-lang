// DMOPC '20 Contest 4 P3 Roving Roombas: a sweeper cleans the floor from
// (0, y) to (x, y) and a climber cleans the wall from (x, 0) to (x, y). Count
// the pairs of one sweeper and one climber whose paths cross.
// original: corpus/submissions/3484973.cpp
type Pt = (x: 1..=1e9, y: 1..=1e9)
let n: 1..=2e5 = read()
let m: 1..=2e5 = read()
let sweepers: [n]Pt = read()
let climbers: [m]Pt = read()

// A climber crosses a sweeper when it stands at or before the sweeper's end
// and climbs at least as high. Go down the heights: every climber met so far
// is tall enough, so a sweeper counts those standing at or before its end.
type Stop = (y: Int, sweeps: Bool, x: Int)
var stops: List<Stop> = []
for c in climbers: stops.push((c.y, false, c.x))
for s in sweepers: stops.push((s.y, true, s.x))
sort(mut stops, by: s => (-s.y, s.sweeps))  // at one height the climbers come first

let xs = [c.x for c in climbers]
let (X, col) = compress(xs)
var standing = SegTree<X, Int>(+, fill: 0)
var crossings = 0
for s in stops:
    if s.sweeps:
        if let reach = col.last_le(s.x): crossings += standing.fold(X.first!..=reach)
    else:
        let at = col.first_ge(s.x)!
        standing.set(at, standing.get(at) + 1)
print(crossings)
