// UCup 4 Stage 25 (Zagreb) E: divide-and-conquer Minkowski sum over prefix-sum
// upper convex hulls to find maximum subarray sum per length, then answer offline
// linear queries on the resulting upper hull.
struct Pt(x: Int, y: Int)

fn add(p: Pt, r: Pt) -> Pt = Pt(p.x + r.x, p.y + r.y)
fn sub(p: Pt, r: Pt) -> Pt = Pt(p.x - r.x, p.y - r.y)
fn cross(p: Pt, r: Pt) -> Int = p.x * r.y - p.y * r.x
fn cross3(p: Pt, r: Pt, s: Pt) -> Int = cross(sub(r, p), sub(s, p))
fn dot(p: Pt, r: Pt) -> Int = p.x * r.x + p.y * r.y

fn upper_hull(pts: List<Pt>) -> List<Pt>:
    var res: List<Pt> = []
    for p in pts:
        while res.len >= 2:
            let top = res.pop()!
            if cross3(res.last!, top, p) < 0:
                res.push(top)
                break
        res.push(p)
    res

@! multitest
let (n, q): (Int, Int) = read()
let a: [0..n]Int = read()

var pref: [0..=n]Pt = Pt(0, 0)
for i in 0..n: pref[i + 1] = Pt(i + 1, pref[i].y + a[i])
var maxs: [0..n]Pt = [0..n] i => pref[i + 1]

fn try_max(p: Pt): maxs[p.x - 1].y max= p.y

fn solve_dc(lo: Int, hi: Int):
    if hi - lo <= 1: return
    let md = (lo + hi) / 2
    solve_dc(lo, md)
    solve_dc(md, hi)
    let los = upper_hull([Pt(-pref[i].x, -pref[i].y) for i in (lo..md).rev])
    let his = upper_hull([pref[i] for i in md..hi])
    var (i, j) = (0, 0)
    try_max(add(los[0], his[0]))
    while i + 1 < los.len or j + 1 < his.len:
        let step_i = i + 1 < los.len and (
            j + 1 == his.len or cross(sub(los[i + 1], los[i]), sub(his[j + 1], his[j])) < 0
        )
        if step_i: i += 1 else: j += 1
        try_max(add(los[i], his[j]))

solve_dc(0, n + 1)
let hull = upper_hull([maxs[i] for i in 0..n])

let qs_in: [0..q]Int = read()
let queries = [(qs_in[idx], idx) for idx in 0..q].sorted()
var ans: [0..q]Int = 0
var hi_idx = 0
for (v, q_id) in queries:
    let cnd = Pt(v, 1)
    while hi_idx + 1 < hull.len and dot(sub(hull[hi_idx + 1], hull[hi_idx]), cnd) >= 0:
        hi_idx += 1
    ans[q_id] = dot(hull[hi_idx], cnd)
print(ans)
