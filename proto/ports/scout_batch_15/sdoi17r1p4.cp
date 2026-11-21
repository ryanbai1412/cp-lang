// SDOI 2017 Ball Dance: n boys and n girls; pairing boy i with girl j gives
// numerator a[i][j] and denominator b[i][j]. Find a perfect bipartite matching
// maximizing sum(a) / sum(b), printed to 6 decimal places.
//
// Fractional programming: ratio >= r iff max_matching(a[i][j] - r * b[i][j]) >= 0.
// Binary search `r` scaled by 10^7 using `scout.flow_ext.max_weight_matching`
// (O(n^3) Hungarian), replacing the original's 90-line `__int128` dense flow.
// Bounding `hi` by `max(a) * SCALE` avoids 128-bit arithmetic completely.
// original: corpus/submissions/5466798.cpp
use scout.flow_ext.{max_weight_matching}

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
    let best = max_weight_matching::<Boy, Girl>((i, j) => SCALE * a[i][j] - mid * b[i][j]).total
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
