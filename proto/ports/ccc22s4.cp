// CCC 2022 S4 Good Triplets: n points sit on c evenly spaced spots of a
// circle. Count the triples of points whose triangle has the centre strictly
// inside.
use std.math.{choose}

let (n, c): (3..=1e6, 3..=1e6) = read()
domain Spot = 0..c
let points: [0..n]Spot = read()

var at: [Spot]Int = 0
for p in points: at[p] += 1
domain Twice = 0..2 * c                     // the circle unrolled twice
let ring = Prefix1<Twice>([Twice] t => at[Spot(Int(t) % c)])

// A triple is bad when it fits in a closed half circle. Charge it to its
// first spot going clockwise: the other two points are there too, or within
// the next c / 2 spots.
var bad = 0
for p in Spot:
    let next = Twice(Int(p) + 1)
    let ahead = ring.sum(next.l..next.r(c / 2))
    bad += choose(at[p], 3) + choose(at[p], 2) * ahead + at[p] * choose(ahead, 2)
// When c is even, a triple with two points on one end of a diameter and one
// on the other was charged at both ends.
if c % 2 == 0:
    for p in Spot.first..Spot(c / 2):
        let q = p.cyc(c / 2)
        bad -= choose(at[p], 2) * at[q] + at[p] * choose(at[q], 2)
print(choose(n, 3) - bad)
