// expect: ok, output "ok"
// input: "1 300\n"
// model: Step
// backtrack's Steps over seeded random trees, pruning a seeded random set of states, against a
// recursive preorder walk that skips the same subtrees: states and depths compared, inside a fn
// and at top level. Reads `seed n`; prints the first tree that differs.
use std.dp.{backtrack}

fn kids(x: Int, w: Int, m: Int) -> List<Int> = if x < m: [x * w + j for j in 1..=w] else: []

fn brute(x: Int, d: Int, w: Int, m: Int, cut: Int, mut out: List<(Int, Int)>):
    out.push((x, d))
    if x % cut == 0: return
    for y in kids(x, w, m): brute(y, d + 1, w, m, cut, mut out)

fn walk(w: Int, m: Int, cut: Int) -> List<(Int, Int)>:
    var out: List<(Int, Int)> = []
    for s in backtrack(0, x => kids(x, w, m)):
        out.push((s.state, s.depth))
        if s.state % cut == 0 and s.depth > 0: s.prune()
    out

let seed0 = read<Int>()
let n = read<Int>()
var seed = seed0
var bad = ""
for k in 0..n:
    seed = (seed * 1103515245 + 12345) % 2147483648
    let w = 1 + seed % 3
    let m = seed / 3 % 40
    let cut = 2 + seed / 120 % 5
    var want: List<(Int, Int)> = [(0, 0)]
    for y in kids(0, w, m): brute(y, 1, w, m, cut, mut want)
    var top: List<(Int, Int)> = []
    for s in backtrack(0, x => kids(x, w, m)):
        top.push((s.state, s.depth))
        if s.state % cut == 0 and s.depth > 0: s.prune()
    if walk(w, m, cut) != want:
        bad = "op {k}: in a fn, width {w}, below {m}, cut {cut}"
    elif top != want:
        bad = "op {k}: at top level, width {w}, below {m}, cut {cut}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
