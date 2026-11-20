// IOI 2015 Day 2 P1, Horses: for n years, the herd multiplies by x[i] in
// year i and a horse then sells for y[i]. Starting from one horse and
// selling everything in the best single year gives max over i of
// x[1] * ... * x[i] * y[i]; print it modulo 1e9 + 7 at the start and
// after each update of one x or one y.
//
// Walk the years backwards in blocks that end at a year with x > 1,
// keeping the best sale so far as a multiple of the herd at the block
// start. Once that multiple reaches 1e9 no earlier year can win, and
// each block at least doubles it, so about 30 blocks are visited.
// input: n, x[], y[], m, then m lines `kind year value` (0-based year;
// kind 1 sets x, kind 2 sets y)
// original: corpus/submissions/5723351.cpp (the three grader functions)
use std.mod.{Mod}
use std.segtree.{SegTree}
use std.algebra.{Max, Prod}
type Z = Mod<1_000_000_007>

let n: Int = read()
domain Year = 0..n
var x: [Year]Int = read()
let y: [Year]Int = read()
var growth = SegTree<Year, Z>.of(Prod<Z>, i => Z(x[i]))
var surge = SegTree<Year, Int>.from(Max<Int>, x)       // to find the years with x > 1
var price = SegTree<Year, Int>.from(Max<Int>, y)

fn best() -> Z:
    var (lead, ans) = (0, Z(0))     // best sale / herd at `end`, and the sale itself
    var end = Year.bounds.last
    while lead < 1_000_000_000 and end != Year.bounds.first:
        let quiet = surge.min_left(end, s => s <= 1)
        let start = quiet.left ?? Year.first
        let top = price.fold(start.l..end)
        if top > lead:
            lead = top
            ans = growth.fold(Year.bounds.first..start.r) * Z(top)
        lead *= x[start]
        end = start.l
    ans

print(best())
let m: Int = read()
for _ in 0..m:
    let (kind, at, value): (Int, Year, Int) = read()
    if kind == 1:
        x[at] = value
        growth.refresh(at)
        surge.set(at, value)
    else:
        price.set(at, value)
    print(best())
