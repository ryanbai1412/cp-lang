// CCC '20 S1 (Surmising a Sprinter's Speed): maximum average speed between consecutive
// sorted (time, position) observations, formatted to 5 decimal places.
// Exercises v2.1 F64 explicit casts, .adjacent, max(gen) -> ?F64, and "{ans:.5f}".
// original: corpus/submissions/4323572.cpp

let n: 2..=100_000 = read()
var obs: [0..n](t: Int, x: Int) = read()
sort(mut obs)
let ans = max(F64(abs(b.x - a.x)) / F64(b.t - a.t) for (a, b) in obs.adjacent)!
print("{ans:.5f}")
