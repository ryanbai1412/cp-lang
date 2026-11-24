// Source: IOI '07 P1 (Aliens), https://dmoj.ca/problem/ioi07p1
// limits: 1000 ms, 256 MB
// Problem: interactive. An N x N meadow (15 <= N <= 2e9) holds a crop formation: 13 flattened
//   M x M squares (M odd, unknown) in a 5 x 5 checkerboard. Given N and one flattened cell
//   (X0, Y0), ask `examine X Y` (true if flattened) at most 300 times and print
//   `solution XC YC`, the formation's centre cell.
// Solution: `reach` finds how far the flattened run goes in one direction by doubling then
//   binary search (O(log N) asks); left and right give the square side m and the left edge,
//   down gives the bottom edge. Hop diagonally by m, then by 2m left and down, while still
//   flattened, to reach the bottom-left square; the centre is that corner + 2m + m / 2.
// original: corpus/submissions/5111683.cpp
// Bounds: n: 15..=2e9; x0, y0: 1..=n
let n: 15..=2e9 = read()
let x0: 1..=n = read()
let y0: 1..=n = read()

fn examine(x: Int, y: Int) -> Bool:
    if x < 1 or x > n or y < 1 or y > n:
        return false
    let rep: Str = ask("examine", x, y)
    rep == "true"

fn reach(dx: Int, dy: Int) -> Int:
    var lo = 0
    var hi = 1
    while examine(x0 + dx * hi, y0 + dy * hi):
        lo = hi
        hi *= 2
    while lo < hi:
        let mid = (lo + hi + 1) / 2
        if examine(x0 + dx * mid, y0 + dy * mid):
            lo = mid
        else:
            hi = mid - 1
    lo

let xl0 = x0 - reach(-1, 0)
let xr0 = x0 + reach(1, 0)
let m = xr0 - xl0 + 1
let yl0 = y0 - reach(0, -1)

var xl = xl0
var yl = yl0
while examine(xl - m, yl - m):
    xl -= m
    yl -= m
while examine(xl - 2 * m, yl):
    xl -= 2 * m
while examine(xl, yl - 2 * m):
    yl -= 2 * m

print("solution", xl + 2 * m + m / 2, yl + 2 * m + m / 2)
