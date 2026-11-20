// DMOPC '20 Contest 4 P3 Roving Roombas: a sweeper cleans the floor from
// (0, y) to (x, y) and a climber cleans the wall from (x, 0) to (x, y). Count
// the pairs of one sweeper and one climber whose paths cross.
// original: corpus/submissions/3484973.cpp
type Pt = (x: 1..=1e9, y: 1..=1e9)
let (n, m): (1..=2e5, 1..=2e5) = read()                                             // WISH S8
let sweepers: [n]Pt = read()
let climbers: [m]Pt = read()

// A climber crosses a sweeper when it stands at or before the sweeper's end
// and climbs at least as high. Go down the heights: every climber met so far
// is tall enough, so a sweeper counts those standing at or before its end.
var coming = Deque(sorted(climbers, by: c => c.y))                                  // WISH S12
var standing = SortedBag<Int>()                                                     // WISH S14 (F13, B1, D1)
var crossings = 0
for s in sorted(sweepers, by: s => s.y).rev:
    for c in coming.pop_back_while(c => c.y >= s.y): standing.add(c.x)              // WISH S12
    crossings += standing.count(..=s.x)                                             // WISH S14
print(crossings)
