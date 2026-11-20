// IOI 2007 P1, Aliens — ideal CP Lang sketch
// Category: Interactive (huge 2e9 coordinate space, hidden 5x5 geometry, 300-query limit, constructive gen)
// Original C++: corpus/submissions/5111683.cpp (157 LOC raw, ~95 LOC stripped)
// Buildable today-form: proto/ports/scout_batch_19/today/ioi07p1.cp (verified vs orig C++)

let n: 15..=2e9 = read()
let x0: 1..=n = read()
let y0: 1..=n = read()

fn examine(x: Int, y: Int) -> Bool:
    if x < 1 or x > n or y < 1 or y > n:
        return false
    // WISH B19-1: typed Bool reply with custom true/false words + shared query budget
    ask("examine", x, y, limit: 300) -> Bool(true: "true", false: "false")

// Exponential doubling + binary search along direction (dx, dy)
// NOTE: `extent` clashed with `std::extent` in proto/cpc C++ codegen (friction B19-8)
fn extent(dx: Int, dy: Int) -> Int:
    var lo = 0
    var hi = 1
    while examine(x0 + dx * hi, y0 + dy * hi):
        lo = hi
        hi *= 2
    // WISH: partition_point / bsearch_last over lo..=hi
    bsearch_last(lo..=hi, k => examine(x0 + dx * k, y0 + dy * k))

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

// WISH B19-3: constructive `gen` for coupled public + hidden state (rejection rate of random x0,y0 is ~1/n^2)
// Shrinking operates on the generator's parameters (m, n, bx, by, ri, rj, ox, oy) rather than raw stdin,
// so every shrunk counterexample keeps (x0, y0) inside a valid red square of the 5x5 pattern.
interactor:
    gen (n, x0, y0) with hidden (m, bx, by):
        let m  = rng.int(1..=size / 5)           // odd in original IOI, any >= 1 works in DMOJ
        let n  = rng.int(5 * m..=max(15, size))
        let bx = rng.int(1..=(n - 5 * m + 1))
        let by = rng.int(1..=(n - 5 * m + 1))
        let (ri, rj) = rng.choice([(i, j) for i in 0..5 for j in 0..5 if (i + j) % 2 == 0])
        let x0 = bx + ri * m + rng.int(0..m)
        let y0 = by + rj * m + rng.int(0..m)
        yield (n, x0, y0) with (m, bx, by)

    limit queries <= 300

    on ask("examine", qx: Int, qy: Int) -> Bool:
        require 1 <= qx <= n and 1 <= qy <= n
        let dx = qx - bx
        let dy = qy - by
        0 <= dx < 5 * m and 0 <= dy < 5 * m and ((dx / m) + (dy / m)) % 2 == 0

    check:
        let (tag, sx, sy): (Str, Int, Int) = read_output()
        require tag == "solution"
        require (sx, sy) == (bx + 2 * m + m / 2, by + 2 * m + m / 2)
