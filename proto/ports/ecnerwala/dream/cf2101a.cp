// Codeforces 2101 A: Mex in the Grid (dream).
// To maximize the sum of MEX over all subgrids, place 0..n^2 - 1 outward from
// the center in concentric (k + 1) x (k + 1) shells so every subgrid containing
// the center captures the largest possible prefix 0..m - 1.
@!multitest
let n: 1..=500 = read()
domain Coord = 0..n
var grid: [Coord, Coord]Int = 0

// Maps shell index a in 0..n to coordinates alternating outward from the center.
fn out_from_center(a: Int) -> Coord = Coord(if (a & 1) == 1: n / 2 - 1 - a / 2 else: n / 2 + a / 2)

var order: List<(Coord, Coord)> = []
for k in 0..n:
    let rk = out_from_center(k)
    for j in 0..k: order.push((rk, out_from_center(j)))
    for j in 0..=k: order.push((out_from_center(j), rk))

for (rank, (r, c)) in order.items: grid[r, c] = rank
for r in Coord: print(grid[r, ..])
