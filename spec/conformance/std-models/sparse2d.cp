// expect: ok, output "ok"
// input: "1 300\n"
// model: SparseTable2D
// SparseTable2D (max) against a plain grid, rebuilt after each random cell change; each op checks a
// random rectangle's fold and try_fold (None when empty) and a cell's get against loops. Reads
// `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.sparse.{SparseTable2D}
domain R = 0..6
domain C = 0..9

fn brute(g: List<List<Int>>, r0: Int, r1: Int, c0: Int, c1: Int) -> Int:
    var m = -1
    for i in r0..r1:
        for j in c0..c1: m = max(m, g[i][j])
    m

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var g = [[rng.int(0, 30) for _ in 0..9] for _ in 0..6]
var bad = ""
for k in 0..n:
    let (i, j) = (rng.int(0, 5), rng.int(0, 8))
    g[i][j] = rng.int(0, 30)
    let st = SparseTable2D(max, [R, C] (r, c) => g[r.ord][c.ord])
    let r0 = rng.int(0, 6)
    let r1 = rng.int(r0, 6)
    let c0 = rng.int(0, 9)
    let c1 = rng.int(c0, 9)
    let rows = R.cuts.nth(r0)..R.cuts.nth(r1)
    let cols = C.cuts.nth(c0)..C.cuts.nth(c1)
    if r0 == r1 or c0 == c1:
        if st.try_fold(rows, cols) != None: bad = "op {k}: try_fold {r0}..{r1} x {c0}..{c1} not None"
    elif st.fold(rows, cols) != brute(g, r0, r1, c0, c1) or st.try_fold(rows, cols) != Some(brute(g, r0, r1, c0, c1)):
        bad = "op {k}: fold {r0}..{r1} x {c0}..{c1}"
    if bad == "" and st.get(R(i), C(j)) != g[i][j]:
        bad = "op {k}: get {i} {j}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
