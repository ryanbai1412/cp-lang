// NOI 2014 Getting-Up Syndrome: n gates in a row (n <= 100000), each `AND t`,
// `OR t` or `XOR t` with 0 <= t <= 1e9. Pick a starting number from 0 to m
// (m <= 1e9), pass it through every gate in order, and print the largest
// result.
//
// The gates work on every bit alone, so send all-zeros and all-ones through
// once. Then choose the start from the top bit down: a 0 that already comes
// out as 1 is free; otherwise a 1 is worth it if it comes out as 1 and still
// fits under m.
// original: corpus/submissions/3595714.cpp
let (n, m): (Int, Int) = read()
let bits = 30
var (from_zeros, from_ones) = (0, (1 << bits) - 1)
for _ in 0..n:
    let (gate, t): (Str, Int) = read()
    if gate == "AND": (from_zeros, from_ones) = (from_zeros & t, from_ones & t)
    elif gate == "OR": (from_zeros, from_ones) = (from_zeros | t, from_ones | t)
    else: (from_zeros, from_ones) = (from_zeros ^ t, from_ones ^ t)

var (start, best) = (0, 0)
for b in (0..bits).rev:
    let bit = 1 << b
    if from_zeros & bit != 0: best |= bit
    elif from_ones & bit != 0 and start + bit <= m:
        start += bit
        best |= bit
print(best)
