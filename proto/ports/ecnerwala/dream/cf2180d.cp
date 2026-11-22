// Codeforces 2180 D — Tangent Disk Chains on a Line
//
// Greedy left-to-right over sorted disk centres `x[0..n]`: maintain the open
// interval `(r_mi, r_ma)` of feasible positive radii for disk `i - 1` within
// the current tangent chain, and `v` as the number of tangent pairs formed.
//
// Extending the chain across gap `d = x[i] - x[i - 1]` requires `r_{i - 1} < d`
// (clamping `r_ma min= d`) and sets `r_i = d - r_{i - 1}`, which reflects the
// feasible open interval `(r_mi, r_ma)` to `(d - r_ma, d - r_mi)`. If clamping
// empties the interval (`r_ma <= r_mi`), no tangent chain survives gap `d`, so
// we start a fresh chain at disk `i` with feasible interval `(0, d)`;
// otherwise we add the tangent pair (`v += 1`) and carry the reflected
// interval forward.

@!multitest
let n: 1..=200_000 = read()
let x: [0..n]0..=10 ** 9 = read()
if n == 1:
    print(0)
    return

var v = 1
var (r_mi, r_ma) = (0, x[1] - x[0])
for i in 2..n:
    let d = x[i] - x[i - 1]
    r_ma min= d
    if r_ma <= r_mi:
        (r_mi, r_ma) = (0, d)
    else:
        v += 1
        (r_mi, r_ma) = (d - r_ma, d - r_mi)
print(v)
