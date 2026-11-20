// COI 2007 #1: n people stand on a line, left to right, and each may walk
// at most t. Afterwards every person must stand at most k to the right of
// the one before. Print the least t, to three decimals.
//
// Sweep left to right keeping p, the rightmost place the previous person can
// end. A person who cannot reach p + k even by walking t right is fine and
// ends at x + t. One who is still beyond p + k after walking t left needs
// more time: the gap is closed from both sides, everyone before shifting
// right by the same extra time.
// original: corpus/submissions/5323995.cpp
let (k, n): (F64, 1..=1e5) = read()
let xs: [0..n]F64 = read()

var t = 0.0
var p = xs[0] - k
for x in xs:
    if abs(p + k - x) <= t: p += k
    elif x <= p + k: p = x + t
    else:
        let more = (x - t - p - k) / 2
        p = x - t - more
        t += more
print("{t:.3f}")
