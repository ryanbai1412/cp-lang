// Source: AtCoder Educational DP Contest Y (Grid 2), https://atcoder.jp/contests/dp/tasks/dp_y
// limits: 2000 ms, 1024 MB
// Problem: an H x W grid (H, W <= 1e5) with N <= 3000 wall cells. Count the right/down paths
//   from (1, 1) to (H, W) that avoid the walls, mod 1e9+7.
// Solution: sort the walls; clean[i] = paths to wall i through no earlier wall = paths(start,
//   i) minus the sum of clean[j] * paths(j, i) over earlier walls j, each a binomial
//   C(dr + dc, dr) from `Comb`. The answer is paths(start, goal) minus the sum of
//   clean[i] * paths(i, goal): O(N^2).
// Bounds: h, w: 2..=1e5; n: 1..=3000
type Z = Mod<1_000_000_007>
let (h, w, n): (2..=1e5, 2..=1e5, 1..=3000) = read()
type Cell = (r: 1..=h, c: 1..=w)
domain Wall = 0..n
var walls: [Wall]Cell = read()
walls.sort()

let comb = Comb<Z>(h + w)
fn paths(a: Cell, b: Cell) -> Z:
    if b.r < a.r or b.c < a.c: return 0
    comb.choose(b.r - a.r + b.c - a.c, b.r - a.r)

let (start, goal): (Cell, Cell) = ((1, 1), (h, w))
var clean: [Wall]Z = 0
for i in Wall:
    clean[i] = paths(start, walls[i]) - sum(clean[j] * paths(walls[j], walls[i]) for j in ..i)
print(paths(start, goal) - sum(clean[i] * paths(walls[i], goal) for i in Wall))
