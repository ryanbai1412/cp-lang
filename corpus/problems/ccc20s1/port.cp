// Source: CCC 2020 S1 (Surmising a Sprinter's Speed), https://dmoj.ca/problem/ccc20s1
// Problem: n <= 1e5 observations (t, x) of a sprinter, times distinct, |t|, |x| <= 1e9.
//   Print the largest speed he certainly reached (relative error 1e-5).
// Solution: sorted by time, the answer is the largest |dx| / dt between neighbours. Keep the
//   speeds as exact `Ratio`s and print the largest rounded to 5 decimals with `fixed` (no
//   floats). O(n log n).
// original: corpus/submissions/4323572.cpp
// Bounds: n: 2..=1e5; seen.t, seen.x: -1e9..=1e9
let n: 2..=1e5 = read()
let seen: [n](t: -1e9..=1e9, x: -1e9..=1e9) = read()
let speeds = [(dx: abs(b.x - a.x), dt: b.t - a.t) for (a, b) in seen.sorted(by: o => o.t).adjacent]
let top = speeds.max(by: s => Ratio(s.dx, s.dt))!
print(fixed(top.dx, top.dt, 5))
