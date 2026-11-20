// An Animal Contest 4 P3: n points and m slopes a / b. Count the pairs of
// points that lie on a common line with one of the slopes; a pair counts
// once for each distinct slope it fits.
let n: 1..=2000 = read()
let m: 1..=2000 = read()
let points: [n](x: 1..=1e9, y: 1..=1e9) = read()
let slopes: [m](a: 1..=1e9, b: 1..=1e9) = read()

// 2/4 and 1/2 are one slope
let distinct = Set(Frac(s.a, s.b) for s in slopes)                  // WISH G10: a reduced fraction; G5: Set from a generator

// two points fit a slope a / b when b * y - a * x agrees
print(sum(equal_pairs(f.den * p.y - f.num * p.x for p in points) for f in distinct))   // WISH G11
