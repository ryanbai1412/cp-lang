// Source: NOI 2014 P1 (Getting-Up Syndrome), https://dmoj.ca/problem/noi14p1
// Problem: n <= 1e5 gates, each AND t, OR t or XOR t (t <= 1e9). Choose a start 0 <= x <= m
//   (m <= 1e9), pass it through all gates in order, and print the largest result.
// Solution: the gates act on each bit alone, so send all-zeros and all-ones through once to
//   see what each bit becomes from 0 and from 1. Then greedily from the top bit: a 0 that
//   comes out 1 is free; else take a 1 if it comes out 1 and the start stays <= m. O(n + 30).
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
