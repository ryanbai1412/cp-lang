// Source: AtCoder Educational DP Contest Y (Grid 2), https://atcoder.jp/contests/dp/tasks/dp_y
// Problem: an H x W grid (H, W <= 1e5) with N <= 3000 wall cells. Count the right/down paths
//   from (1, 1) to (H, W) that avoid the walls, mod 1e9+7.
// Solution: sort the walls; clean[i] = paths to wall i through no earlier wall = paths(start,
//   i) minus the sum of clean[j] * paths(j, i) over earlier walls j, each a binomial from
//   factorial tables. The answer is paths(start, goal) minus the sum of
//   clean[i] * paths(i, goal): O(N^2).
const P = 1000000007
type Z = Mod<P>
let h: 2..=1e5 = read()
let w: 2..=1e5 = read()
let n: 1..=3000 = read()
type Cell = (r: 1..=h, c: 1..=w)
let start: Cell = (1, 1)
let goal: Cell = (h, w)
domain W = 0..n
let walls: [W]Cell = read()
assert walls.sorted().deduped().len == n and all(c != start and c != goal for c in walls)

domain F = 0..=200000
var fact: [F]Z = 1
for f in F:
    if let p = f.prev: fact[f] = fact[p] * Z(f.label)

var inv_fact: [F]Z = Z.one / fact[F.last!]
for f in F.rev:
    if let p = f.prev: inv_fact[p] = inv_fact[f] * Z(f.label)

fn paths(a: Cell, b: Cell) -> Z:
    if b.r < a.r or b.c < a.c: return 0
    let (down, right) = (b.r - a.r, b.c - a.c)
    fact[F(down + right)] * inv_fact[F(down)] * inv_fact[F(right)]

let by_pos = walls.sorted()
let order: [W]Cell = [W] i => by_pos[i.ord]
var clean: [W]Z = 0
for i in W:
    clean[i] = paths(start, order[i]) - sum(clean[j] * paths(order[j], order[i]) for j in ..i)
print(paths(start, goal) - sum(clean[i] * paths(order[i], goal) for i in W))
