// IOI 2007 P1, Aliens (interactive, buildable today-form)
// original: corpus/submissions/5111683.cpp
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
