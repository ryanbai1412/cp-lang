// Codeforces 2022 E1: a grid is beautiful iff every cell is row[r] xor col[c] for some row and column labels.
// A fixed cell is one constraint row[r] xor col[c] = v, so rows and columns form a graph in which each
// component has one free 30-bit label, less one for the global xor shift that leaves the grid unchanged.
// A contradictory cycle gives 0; otherwise the count is 2^(30 * (components - 1)).
use std.dsu.PotentialDsu

@!multitest
type Z = Mod<1e9 + 7>
let (n, m, k, _) = read<(2..=1e5, 2..=1e5, 0..=1e5, 0..=0)>()
domain Line = Row(1..=n) | Col(1..=m)
var labels = PotentialDsu<Line>(op: ^)
let cells = read<[k](r: 1..=n, c: 1..=m, v: 0..(1 << 30))>()
let consistent = cells.all((r, c, v) => labels.unite(Row(r), Col(c), v))
print(if consistent: Z(2) ** (30 * (labels.components - 1)) else: 0)
