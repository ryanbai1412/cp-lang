// IOI '22 P1 — Fish (runnable proto/cpc port of ioi22p1_draft.cp, post-Drill-1 pivot)
// Prototype deltas vs v2.1 draft: no `@grader fn`, no `for mut`, no Tropical Optionals,
// explicit `psum` binary search over sorted per-column `(y, prefix_w)` lists.

let n: 2..=100_000 = read()
let m: 1..=300_000 = read()

domain Col = 0..n
var ys: [Col]List<Int> = []
var raw_fish: [Col]List<(Int, Int)> = []

for _ in 0..m:
    let x: Int = read()
    let y: Int = read()
    let w: Int = read()
    raw_fish[Col.from_idx(x)!].push((y, w))

var ps: [Col]List<Int> = []
for c in Col:
    raw_fish[c].sort()
    var run = 0
    for f in raw_fish[c]:
        ys[c].push(f.0)
        run += f.1
        ps[c].push(run)

// Sum of weights in column `col` strictly below row `h` (rows 0..h-1).
fn psum(col: Int, h: Int) -> Int:
    if col < 0 or col >= n:
        return 0
    let c = Col.from_idx(col)!
    let k = ys[c].len
    var lo = 0
    var hi = k
    while lo < hi:
        let mid = (lo + hi) / 2
        if ys[c][mid] >= h:
            hi = mid
        else:
            lo = mid + 1
    if lo == 0:
        return 0
    return ps[c][lo - 1]

fn height_at(col: Int, j: Int) -> Int:
    if col < 0 or col >= n:
        return 0
    let c = Col.from_idx(col)!
    if j < ys[c].len:
        return ys[c][j]
    return n

fn list_max(xs: List<Int>) -> Int:
    var best = xs[0]
    for x in xs:
        if x > best:
            best = x
    return best

// dp0[i][j] = best for suffix i..n-1 with pier i at height_at(i, j), col i's own fish NOT yet swept by col i-1
// dp1[i][j] = best for suffix i..n-1 with pier i at height_at(i, j), allowed to sweep col i from col i+1
var next_dp0: List<Int> = [0]
var next_dp1: List<Int> = [0]

for i in (0..n).rev:
    let c = Col.from_idx(i)!
    let cnt = ys[c].len
    let m2 = next_dp0.len
    let max_n0 = list_max(next_dp0)
    let max_n1 = list_max(next_dp1)
    let base0 = if max_n0 > max_n1: max_n0 else: max_n1

    var cmx_up = 0
    for k in 0..m2:
        let v = next_dp0[k] - psum(i + 1, height_at(i + 1, k))
        if v > cmx_up:
            cmx_up = v

    var cmx_down = 0
    for k in 0..m2:
        let v = next_dp1[k] + psum(i, height_at(i + 1, k))
        if v > cmx_down:
            cmx_down = v

    var cur_dp0: List<Int> = []
    var cur_dp1: List<Int> = []
    for j in 0..=cnt:
        let hj = height_at(i, j)
        let cand_up = cmx_up + psum(i + 1, hj)
        let v0 = if base0 > cand_up: base0 else: cand_up
        cur_dp0.push(v0)

        var v1 = max_n1
        let cand_down = cmx_down - psum(i, hj)
        if cand_down > v1:
            v1 = cand_down
        if cand_up > v1:
            v1 = cand_up
        cur_dp1.push(v1)

    next_dp0 = cur_dp0
    next_dp1 = cur_dp1

print(list_max(next_dp1))
