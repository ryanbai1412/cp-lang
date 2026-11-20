// cp: 0.1 1f59ac0+dirty
// AtCoder DP Y Grid 2: count monotone paths from (1,1) to (h,w) avoiding n
// walls. clean[c] = paths reaching cell c touching no wall before it:
// all paths minus those whose first wall is an earlier cell.
const P = 1000000007
type Z = Mod<P>
let h: 2..=1e5 = read()
let w: 2..=1e5 = read()
let n: 1..=3000 = read()
type Cell = (r: 1..=h, c: 1..=w)
let start: Cell = (1, 1)
let goal: Cell = (h, w)
domain W = 0..n
let walls: [W]Cell = read() where distinct(walls) and all(c != start and c != goal for c in walls)

domain F = 0..=200000
var fact: [F]Z = Z(1)
for f in F:
    if let p = f.prev: fact[f] = fact[p] * Z(f.label)

var inv_fact: [F]Z = Z(1) / fact[F.last]
for f in F.rev:
    if let p = f.prev: inv_fact[p] = inv_fact[f] * Z(f.label)

fn paths(a: Cell, b: Cell) -> Z:
    if b.r < a.r or b.c < a.c: return Z(0)
    let (down, right) = (b.r - a.r, b.c - a.c)
    fact[F(down + right)] * inv_fact[F(down)] * inv_fact[F(right)]

let order = sorted(walls)
var clean: [W]Z = Z(0)
for i in W:
    clean[i] = paths(start, order[i]) - sum(clean[j] * paths(order[j], order[i]) for j in W.first..i)
print(paths(start, goal) - sum(clean[i] * paths(order[i], goal) for i in W))
