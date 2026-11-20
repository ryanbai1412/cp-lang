// IOI 2007 P1, Aliens — CP Lang 0.2.2
// Category: Interactive (huge 2e9 coordinate space, hidden 5x5 geometry, 300-query limit)
// Original C++: corpus/submissions/5111683.cpp (157 LOC raw, ~95 LOC stripped)
// Buildable today-form: proto/ports/scout_batch_19/today/ioi07p1.cp (verified vs orig C++)


let n: 15..=2e9 = read()
let x0: 1..=n = read()
let y0: 1..=n = read()

fn examine(x: Int, y: Int) -> Bool:
    if x < 1 or x > n or y < 1 or y > n:
        return false
    let reply: Str = ask("examine", x, y, limit: 300)
    reply == "true"

// Exponential doubling + binary search along direction (dx, dy)
fn extent(dx: Int, dy: Int) -> Int:
    var lo = 0
    var hi = 1
    while examine(x0 + dx * hi, y0 + dy * hi):
        lo = hi
        hi *= 2
    bsearch_last(lo..=hi, k => examine(x0 + dx * k, y0 + dy * k))!

let xl0 = x0 - extent(-1, 0)
let xr0 = x0 + extent(1, 0)
let m = xr0 - xl0 + 1
let yl0 = y0 - extent(0, -1)

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

@interactor
fn judge(mut sol: Interactor):
    // Hidden test: n, x0, y0, m, bx, by (where (x0, y0) lies in a red square of the 5x5 grid at (bx, by))
    let (n, x0, y0, m, bx, by): (Int, Int, Int, Int, Int, Int) = read()
    sol.send(n, x0, y0)
    var queries = 0
    while true:
        let tag: Str = sol.recv()
        if tag == "examine":
            let (qx, qy): (Int, Int) = sol.recv()
            queries += 1
            assert queries <= 300 and qx in 1..=n and qy in 1..=n
            let dx = qx - bx
            let dy = qy - by
            let red = dx >= 0 and dx < 5 * m and dy >= 0 and dy < 5 * m and ((dx / m) + (dy / m)) % 2 == 0
            sol.send(if red: "true" else: "false")
        elif tag == "solution":
            let (sx, sy): (Int, Int) = sol.recv()
            assert sx == bx + 2 * m + m / 2 and sy == by + 2 * m + m / 2
            return
        else:
            assert false
