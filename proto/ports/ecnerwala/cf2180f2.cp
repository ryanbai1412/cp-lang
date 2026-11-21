// Codeforces 2180 F2: Control Car (Hard Version).
// Follow the car through an unbounded grid, choosing each corner only when the
// car first needs it. In a cell, two inherited corners matter: the floor is
// shut when the bottom-left corner points right, the right wall when the
// top-right one points down. The fresh bottom-right corner decides the move:
// pointing left shuts the floor, pointing up shuts the wall. A fall hands it
// on as the top-right corner below, a slide as the bottom-left corner on the
// right, and the other corner of the new cell is fresh. The car stops inside
// the n x m grid iff it stops in a cell (i, j) with i < n, j < m. For a fixed
// n the count is a linear recurrence in m: tabulate 600 columns and extend it.
use std.spec.{guess_at}
type Z = Mod<1_000_000_007>

domain I = 0..50
domain J = 0..600

// ways[j]: corner choices so far that bring the car into (i, j), split by
// which of floor and wall are shut.
type Ways = (open: Z, walled: Z, shut: Z, boxed: Z)
var ways: [J]Ways = (0, 0, 0, 0)
ways[J.first] = (9, 3, 3, 1)
// A stop in (i, j) has chosen 2(i + j) + 3 corners.
let quarter = Z(1) / 4
let weight = [quarter ** (2 * d + 3) for d in 0..I.len + J.len]
var stopped: [I, J]Z = 0
for i in I:
    var side: Ways = (0, 0, 0, 0)
    var run: Z = 0
    for j in J:
        let w = ways[j]
        let (open, walled, shut) = (w.open + side.open, w.walled + side.walled, w.shut + side.shut)
        run += (walled + shut + 4 * (w.boxed + side.boxed)) * weight[i.to_idx + j.to_idx]
        let falls = open + walled
        ways[j] = (6 * falls, 3 * falls, 2 * falls, falls)
        let slides = open + 2 * shut
        side = (3 * slides, slides, 3 * shut, shut)
        stopped[i, j] = run + (if i == I.first: 0 else: stopped[(i - 1)!, j])

@!multitest
let (n, m): (1..=50, 1..=1e15) = read()
let i = I(n - 1)
let counts = [Z(4) ** ((n + 1) * (j.to_idx + 2)) * stopped[i, j] for j in J]
print(guess_at(counts, m - 1)!)
