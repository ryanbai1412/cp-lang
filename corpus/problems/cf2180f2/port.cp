// Source: Codeforces 2180F2 (Control Car, Hard Version),
//   https://codeforces.com/contest/2180/problem/F2
// Problem: each of the (n+1)(m+1) grid points of an n x m grid gets a wall of length 1 pointing
//   up, down, left or right. A car starts in cell (1, 1), moves down when it can, else right,
//   and stops when blocked or out of the grid. Count orientations where it stops inside the
//   grid, mod 1e9+7. n <= 50 (sum n <= 50), m <= 1e15; multitest.
// Solution: follow the car through an unbounded grid, fixing each corner only when first
//   needed: a cell's bottom-left and top-right corners are inherited, its bottom-right is fresh
//   and decides fall, slide or stop. A row-by-row DP over 50 x 600 cells keeps, per column, the
//   weight of ways to enter split by floor/wall shut, and sums stops in rows ..= i, columns ..= j
//   (weighted by 4^-(corners fixed)). For fixed n the count is a linear recurrence in m:
//   `guess_at` extends the 600 tabulated columns to column m - 1.
use std.spec.{guess_at}
type Z = Mod<1_000_000_007>

domain I = 0..50
domain J = 0..600

// ways[j]: corner choices so far that bring the car into (i, j), split by
// which of floor and wall are shut. Different corners decide the floor and
// the wall: `open` ways reach the floor's corner, which has `floor` ways to
// keep it open, and `shut` ways arrive with it shut; the wall's corner has
// `wall` ways to stay open. Each shut corner has one way.
type Ways = (open: Z, walled: Z, shut: Z, boxed: Z)
fn state(open: Z, shut: Z, floor: Int, wall: Int) -> Ways =
    (open * floor * wall, open * floor, shut * wall, shut)
var ways: [J]Ways = (0, 0, 0, 0)
ways[J.first!] = state(1, 1, floor: 3, wall: 3)
// A stop in (i, j) has chosen 2(i + j) + 3 corners.
let weight = [(Z(1) / 4) ** (2 * d + 3) for d in 0..I.len + J.len]
// stopped[i, j]: stops in rows ..=i and columns ..=j; column[j] its last row.
var stopped: [I, J]Z = 0
var column: [J]Z = 0
for i in I:
    var side: Ways = (0, 0, 0, 0)
    var run: Z = 0
    for j in J:
        let w = ways[j]
        let (open, walled, shut) = (w.open + side.open, w.walled + side.walled, w.shut + side.shut)
        run += (walled + shut + 4 * (w.boxed + side.boxed)) * weight[i.to_idx + j.to_idx]
        let falls = open + walled
        ways[j] = state(falls, falls, floor: 3, wall: 2)
        let slides = open + 2 * shut
        side = state(slides, shut, floor: 1, wall: 3)
        column[j] += run
        stopped[i, j] = column[j]

@!multitest
let (n, m): (1..=50, 1..=1e15) = read()
print(guess_at([Z(4) ** ((n + 1) * (j.to_idx + 2)) * stopped[I(n - 1), j] for j in J], m - 1)!)
