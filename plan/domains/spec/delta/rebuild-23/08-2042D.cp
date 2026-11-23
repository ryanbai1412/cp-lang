// CF 2042D Recommendations: the predictors of i are the segments containing it. Answer
// (min r over predictors) - (max l over predictors) - (r_i - l_i); identical segments give 0.
// Sweep by (l asc, r desc) folding min r over r >= r_i; the mirror (-r, -l) gives max l.
@!multitest
let n = read<1..=2e5>()
domain U = [n]
let seg = read<[U](l: Int, r: Int)>()
let same = seg.tally(by: g => (g.l, g.r))

fn min_cover_r(g: [U](l: Int, r: Int)) -> [U]?Int:
    let (domain C, rs) = compress([g[i].r for i in U])
    var st = SegTree<C, Int>(min, fill: Int.max)
    var out: [U]?Int = None
    for i in U.sorted(by: i => (g[i].l, desc(g[i].r))):
        let c = rs.at(g[i].r)
        let best = st.fold(c..)
        if best != Int.max: out[i] = best
        st.set(c, min(st.get(c), g[i].r))
    out

let hi = min_cover_r(seg)
let lo = min_cover_r([U] i => (l: -seg[i].r, r: -seg[i].l))
let ans = [U] i =>
    if same[(seg[i].l, seg[i].r)] > 1: 0
    elif let r = hi[i], let l = lo[i]: r + l - (seg[i].r - seg[i].l)
    else: 0
print(ans, sep: "\n")
