// Source: DMOPC '20 Contest 4 P3 (Roving Roombas), https://dmoj.ca/problem/dmopc20c4p3
// Problem: n <= 2e5 Roombas clean (0, Y_i) to (X_i, Y_i) and m <= 2e5 cords run (X_j, 0) to
//   (X_j, Y_j), coordinates 1..=1e9 (Roomba Y distinct, cord X distinct). Count the
//   (Roomba, cord) pairs that touch, ends included.
// Solution: a cord crosses a Roomba when X_j <= X_i and Y_j >= Y_i. Sweep heights downward,
//   cords before Roombas at equal height, adding each cord to a SegTree over compressed cord x;
//   each Roomba adds the count of cords at or left of its X_i. O((n + m) log m).
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
