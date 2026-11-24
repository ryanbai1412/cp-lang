// Tuned: one pass over the sorted sightings compares each leg by cross products,
// with no list of legs and no reduced Ratio (a gcd) per leg; sightings are a struct, whose
// range fields are 32-bit where a named tuple's are 64 (sent to Perf).
//
// Source: CCC 2020 S1 (Surmising a Sprinter's Speed), https://dmoj.ca/problem/ccc20s1
// Problem: n <= 1e5 observations (t, x) of a sprinter, times distinct, |t|, |x| <= 1e9.
//   Print the largest speed he certainly reached (relative error 1e-5).
// Solution: sorted by time, the answer is the largest |dx| / dt between neighbours. Keep the
//   speeds as exact `Ratio`s and print the largest rounded to 5 decimals with `fixed` (no
//   floats). O(n log n).
// original: corpus/submissions/4323572.cpp
// Bounds: n: 2..=1e5; seen.t, seen.x: -1e9..=1e9
let n: 2..=1e5 = read()
struct Sighting(t: -1e9..=1e9, x: -1e9..=1e9)
let seen: [n]Sighting = read()
let legs = seen.sorted(by: o => o.t)
var (dx, dt) = (0, 1)
// dx and dt stay below 2e9, so the cross products fit an Int
for i in 1..n:
    let (x, y) = (abs(legs[i].x - legs[i - 1].x), legs[i].t - legs[i - 1].t)
    if x * dt > dx * y: (dx, dt) = (x, y)
print(fixed(dx, dt, 5))
