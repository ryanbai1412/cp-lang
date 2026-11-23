// Source: An Animal Contest 4 P3 (Snowy Slopes), https://dmoj.ca/problem/aac4p3
// Problem: n <= 1e5 points with distinct x and distinct y (coordinates 1..=1e9) and m <= 20
//   slopes k_i / d_i (nonzero, |k|, |d| <= 1e9). Count the pairs of points whose line has one
//   of the given slopes.
// Solution: reduce each slope by its gcd and drop repeats. Points on one line of slope a / b
//   share b * y - a * x, so per slope a HashMap counts the earlier points on each line: O(m n).
// Builds; stress OK against 4140246.cpp.
let n: 1..=2000 = read()
let m: 1..=2000 = read()
let points: [n](x: 1..=1e9, y: 1..=1e9) = read()
let slopes: [m](a: 1..=1e9, b: 1..=1e9) = read()
var directions: Set<(Int, Int)> = []
for s in slopes:
    let g = gcd(s.a, s.b)
    if g == 0: continue
    directions.insert((s.a / g, s.b / g))
var pairs = 0
for (a, b) in directions:
    var on_line: HashMap<Int, Int> = []
    for p in points:
        let line = b * p.y - a * p.x
        pairs += on_line.get(line) ?? 0
        on_line[line] += 1
print(pairs)
