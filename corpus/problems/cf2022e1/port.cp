// Source: Codeforces 2022E1 (Billetes MX, easy), https://codeforces.com/contest/2022/problem/E1
// Problem: an n x m grid (n, m <= 1e5) with k <= 1e5 cells fixed to values < 2^30, q = 0
//   (multitest). Count the fillings with values < 2^30 where every subgrid's four corners xor to
//   0, mod 1e9+7.
// Solution: such a grid is row[r] xor col[c] for some labels, so a fixed cell is an edge
//   row[r] xor col[c] = v between n + m lines. A union-find (by size) keeps each line's xor to its
//   root; a contradictory cycle gives 0. Otherwise each component has a free 30-bit label, less
//   one for the global xor shift: 2^(30 (components - 1)). O((n + m + k) log).
@!multitest
type Z = Mod<1e9 + 7>
type Label = 0..(1 << 30)
let (n, m, k, _) = read<(2..=1e5, 2..=1e5, 0..=1e5, 0..=0)>()
domain Line = 1..=n + m

var up: [Line]Line = [Line] x => x
var size: [Line]1..=Line.count = 1
var gap: [Line]Label = 0

fn find(x: Line) -> (root: Line, label: Label):
    var y = x
    var label: Label = 0
    while up[y] != y:
        label ^= gap[y]
        y = up[y]
    (root: y, label:)

fn unite(a: Line, b: Line, v: Label) -> Bool:
    let (ra, la) = find(a)
    let (rb, lb) = find(b)
    if ra == rb: return la ^ lb == v
    let (big, small) = if size[ra] >= size[rb]: (ra, rb) else: (rb, ra)
    up[small] = big
    gap[small] = la ^ lb ^ v
    size[big] += size[small]
    true

let cells = read<[k](r: 1..=n, c: 1..=m, v: Label)>()
let consistent = cells.all(x => unite(Line(x.r), Line(n + x.c), x.v))
let parts = Line.count(x => up[x] == x)
print(if consistent: Z(2) ** (30 * (parts - 1)) else: 0)
