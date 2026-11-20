// AtCoder DP Y Grid 2: count the right/down paths from (1, 1) to (h, w) that
// avoid n walls. clean[i]: paths that reach wall i having touched no wall
// before it = all paths there minus those whose first wall is an earlier one.
// The goal is treated as one last wall.
const MOD = 1_000_000_007
type Z = Mod<MOD>
type Cell = (r: Int, c: Int)
let (h, w, n): (2..=1e5, 2..=1e5, 1..=3000) = read()
let walls: [0..n]Cell = read()
let comb = Comb<MOD>()

fn paths(a: Cell, b: Cell) -> Z = comb.choose(b.r - a.r + b.c - a.c, b.r - a.r)

let stops = sorted(walls) + [(h, w)]
var clean: List<Z> = []
for i in 0..=n:
    let cell = stops[i]
    var ways = paths((1, 1), cell)
    for j in 0..i if stops[j].c <= cell.c: ways -= clean[j] * paths(stops[j], cell)
    clean.push(ways)
print(clean.last!)
