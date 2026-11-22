// IOI '22 P1 — Fish (CP Lang 0.2.2): `bsearch_first`, `max=`, and `max(...)!`
// replace manual binary search, `list_max`, and `.from_idx(...)!` boilerplate.
// original: corpus/submissions/4785724.cpp

let (n, m): (2..=100_000, 1..=300_000) = read()
domain Col = 0..n
var raw_fish: [Col]List<(Int, Int)> = []
for _ in 0..m:
    let (x, y, w): (Col, Int, Int) = read()
    raw_fish[x].push((y, w))

var ys: [Col]List<Int> = []
var ps: [Col]List<Int> = []
for c in Col:
    raw_fish[c].sort()
    var run = 0
    for (y, w) in raw_fish[c]:
        ys[c].push(y)
        run += w
        ps[c].push(run)

fn psum(col: Int, h: Int) -> Int:
    if col < 0 or col >= n: return 0
    let c = Col(col)
    let lo = bsearch_first(0..ys[c].len, mid => ys[c][mid] >= h) ?? ys[c].len
    if lo == 0: 0 else: ps[c][lo - 1]

fn height_at(col: Int, j: Int) -> Int:
    if col < 0 or col >= n: return 0
    let c = Col(col)
    if j < ys[c].len: ys[c][j] else: n

var next_dp0: List<Int> = [0]
var next_dp1: List<Int> = [0]

for i in (0..n).rev:
    let c = Col(i)
    let cnt = ys[c].len
    let m2 = next_dp0.len
    let max_n0 = next_dp0.max()!
    let max_n1 = next_dp1.max()!
    let base0 = max(max_n0, max_n1)

    var cmx_up = 0
    var cmx_down = 0
    for k in 0..m2:
        let hk = height_at(i + 1, k)
        cmx_up max= next_dp0[k] - psum(i + 1, hk)
        cmx_down max= next_dp1[k] + psum(i, hk)

    var cur_dp0: List<Int> = []
    var cur_dp1: List<Int> = []
    for j in 0..=cnt:
        let hj = height_at(i, j)
        let cand_up = cmx_up + psum(i + 1, hj)
        cur_dp0.push(max(base0, cand_up))
        var v1 = max_n1
        v1 max= cmx_down - psum(i, hj)
        v1 max= cand_up
        cur_dp1.push(v1)

    next_dp0 = cur_dp0
    next_dp1 = cur_dp1

print(next_dp1.max()!)
