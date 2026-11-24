// Source: Codeforces 2180F2 (Control Car, Hard Version),
//   https://codeforces.com/contest/2180/problem/F2
// Problem: each of the (n+1)(m+1) grid points of an n x m grid gets a wall of length 1 pointing
//   up, down, left or right. A car starts in cell (1, 1), moves down when it can, else right,
//   and stops when blocked or out of the grid. Count orientations where it stops inside the
//   grid, mod 1e9+7. n <= 50 (sum n <= 50), m <= 1e15; multitest.
// Solution: F1's sweep (follow the car through an unbounded grid, fixing each corner only when
//   first needed) gives the weighted stops in each cell of a 50 x 600 grid, so the count for an
//   n x k grid is 4^((n+1)(k+1)) times the stops in its first n rows and k columns. For fixed n
//   that count is a linear recurrence in k of order 2n + 1 <= 101 (measured with
//   `berlekamp_massey` for every n), so the 601 tabulated counts pin it and
//   `guess_at` (Berlekamp-Massey, then Kitamasa) extends it to k = m.
// Bounds: n: 1..=I.len; m: 1..=1e15
use std.spec.{guess_at}
type Z = Mod<1_000_000_007>

// ways[j]: corner choices so far that bring the car into (i, j), split by
// which of floor and wall are shut. Different corners decide the floor and
// the wall: `open` ways reach the floor's corner, which has `floor` ways to
// keep it open, and `shut` ways arrive with it shut; the wall's corner has
// `wall` ways to stay open. Each shut corner has one way.
type Ways = (open: Z, walled: Z, shut: Z, boxed: Z)
fn state(open: Z, shut: Z, floor: Int, wall: Int) -> Ways =
    (open * floor * wall, open * floor, shut * wall, shut)

// stops[i, j]: the share of corner choices whose car stops in cell (i, j).
fn stops<domain I, domain J>() -> [I, J]Z:
    var ways: [J]Ways = (0, 0, 0, 0)
    ways[J.first!] = state(1, 1, floor: 3, wall: 3)
    // A stop in (i, j) has chosen 2(i + j) + 3 corners.
    let weight = [(Z(1) / 4) ** (2 * d + 3) for d in 0..I.len + J.len]
    var out: [I, J]Z = 0
    for i in I:
        var side: Ways = (0, 0, 0, 0)
        for j in J:
            let w = ways[j]
            let (open, walled, shut) = (
                w.open + side.open,
                w.walled + side.walled,
                w.shut + side.shut,
            )
            out[i, j] = (walled + shut + 4 * (w.boxed + side.boxed)) * weight[i.ord + j.ord]
            let falls = open + walled
            ways[j] = state(falls, falls, floor: 3, wall: 2)
            side = state(open + 2 * shut, shut, floor: 1, wall: 3)
    out

domain I = 0..50
domain J = 0..600
let inside = Prefix(stops<I, J>())

@!multitest
let (n, m): (1..=I.len, 1..=1e15) = read()
// count[k]: orientations of an n x k grid whose car stops inside it.
let count = [Z(4) ** ((n + 1) * (Int(k) + 1)) * inside.prefix(I.cuts(n), k) for k in J.cuts]
print(guess_at(count, m)!)
