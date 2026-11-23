// Source: SDOI 2017 Round 1 (新生舞会, Freshman Ball), https://dmoj.ca/problem/sdoi17r1p4
//   (not public; statement read on Luogu as P3705)
// Problem: n <= 100 boys and n girls; pairing boy i with girl j gives a[i][j] and b[i][j]
//   (both 1..1e4). Over perfect matchings, maximise sum a / sum b; print it to 6 decimals.
// Solution: fractional programming: the ratio exceeds r iff the max-weight perfect matching
//   with weights a - r b is positive. Binary search r scaled by SCALE = 1e7 (integer weights
//   SCALE a - mid b, hi = max(a) * SCALE) with `max_weight_matching` (Hungarian, O(n^3)),
//   about 37 rounds; print lo / 1e7 rounded to 6 decimals by hand.
// original: corpus/submissions/5466798.cpp
use std.flow.{max_weight_matching}

let n: Int = read()
domain Boy = 0..n
domain Girl = 0..n

let a: [Boy][Girl]Int = read()
let b: [Boy][Girl]Int = read()

const SCALE = 10000000
var max_a = 0
for i in Boy:
    for j in Girl:
        max_a max= a[i][j]

var (lo, hi) = (0, max_a * SCALE)
while lo < hi:
    let mid = (lo + hi + 1) / 2
    let best = max_weight_matching<Boy, Girl>((i, j) => SCALE * a[i][j] - mid * b[i][j]).total
    if best > 0: lo = mid
    else: hi = mid - 1

// Round `lo / 10^7` to 6 decimal places (`print` has no fixed-precision format).
let scaled6 = (lo + 5) / 10
let whole = scaled6 / 1000000
var frac = scaled6 % 1000000
var digits = List.filled(6, 0)
for k in (0..6).rev:
    digits[k] = frac % 10
    frac /= 10
var out = Str(whole) + "."
for d in digits: out += Str(d)
print(out)
