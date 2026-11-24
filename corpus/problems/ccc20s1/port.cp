// Source: CCC 2020 S1 (Surmising a Sprinter's Speed), https://dmoj.ca/problem/ccc20s1
// limits: 1000 ms, 512 MB
// Problem: n <= 1e5 observations (t, x) of a sprinter, times distinct, |t|, |x| <= 1e9.
//   Print the largest speed he certainly reached (relative error 1e-5).
// Solution: sorted by time, the answer is the largest |dx| / dt between neighbours. Keep the
//   speeds as exact `Ratio`s and print the largest rounded to 5 decimals with `fixed` (no
//   floats). O(n log n).
// original: corpus/submissions/4323572.cpp
// Bounds: n: 2..=1e5; seen.t, seen.x: -1e9..=1e9
let n: 2..=1e5 = read()
var seen: [n](t: -1e9..=1e9, x: -1e9..=1e9) = read()
seen.sort(by: o => o.t)
let (a, b) = seen.adjacent.max(by: ((a, b)) => Ratio(abs(b.x - a.x), b.t - a.t))!
print(fixed(abs(b.x - a.x), b.t - a.t, 5))
