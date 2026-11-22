// Codeforces 2180 F2: Control Car (Hard Version) (dream).
// Follow the car through an unbounded grid, choosing each corner only when the
// car first needs it. Two inherited corners say whether the floor and the right
// wall of the cell are shut; the fresh bottom-right corner decides the move.
// A fall hands that corner down as the next top-right one, a slide hands it
// right as the next bottom-left one. A stop in (i, j) has chosen 2(i + j) + 3
// corners; the car stops inside the n x m grid iff it stops at i < n, j < m.
// For fixed n the count is a linear recurrence in m: tabulate and extend it.
use cf2180f1.{Z, stops}

domain I = 0..50
domain J = 0..600
let inside = stops<I, J>().prefix_sums()

@!multitest
let (n, m): (1..=50, 1..=1e15) = read()
print(guess_at([j => 4 ** ((n + 1) * (j + 2)) * inside[..n, ..=j]], at: m - 1)!)
