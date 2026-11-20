// UCup 4 Stage 25 (Zagreb) E: divide-and-conquer Minkowski sum over prefix-sum
// upper convex hulls to find maximum subarray sum per length, then answer offline
// linear queries on the resulting upper hull.
struct Pt:
    x: Int
    y: Int

fn cross(a: Pt, b: Pt) -> Int = a.x * b.y - a.y * b.x
fn cross3(a: Pt, b: Pt, c: Pt) -> Int = cross(Pt(b.x - a.x, b.y - a.y), Pt(c.x - a.x, c.y - a.y))
fn dot(a: Pt, b: Pt) -> Int = a.x * b.x + a.y * b.y

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

@!multitest
let (n, q): (Int, Int) = read()
let a: [0..n]Int = read()

var pref: [0..=n]Pt = Pt(0, 0)
for i in 0..n:
    pref[i + 1] = Pt(i + 1, pref[i].y + a[i])

var maxs: [0..n]Pt = Pt(0, 0)
for i in 0..n: maxs[i] = pref[i + 1]

fn try_max(p: Pt):
    maxs[p.x - 1].y max= p.y

fn solve_dc(lo: Int, hi: Int):
    if hi - lo <= 1: return
    let md = (lo + hi) / 2
    solve_dc(lo, md)
    solve_dc(md, hi)
    let los = upper_hull([Pt(-pref[i].x, -pref[i].y) for i in (lo..md).rev])
    let his = upper_hull([pref[i] for i in md..hi])
    var i = 0
    var j = 0
    try_max(Pt(los[0].x + his[0].x, los[0].y + his[0].y))
    while i + 1 < los.len or j + 1 < his.len:
        let step_i = i + 1 < los.len and (
            j + 1 == his.len or
            cross(
                Pt(los[i + 1].x - los[i].x, los[i + 1].y - los[i].y),
                Pt(his[j + 1].x - his[j].x, his[j + 1].y - his[j].y)
            ) < 0
        )
        if step_i: i += 1
        else: j += 1
        try_max(Pt(los[i].x + his[j].x, los[i].y + his[j].y))

solve_dc(0, n + 1)
let hull = upper_hull([maxs[i] for i in 0..n])

let qs_in: [0..q]Int = read()
let queries = sorted([(qs_in[idx], idx) for idx in 0..q])
var ans: [0..q]Int = 0
var hi_idx = 0
for (v, q_id) in queries:
    let cnd = Pt(v, 1)
    while hi_idx + 1 < hull.len and dot(Pt(hull[hi_idx + 1].x - hull[hi_idx].x, hull[hi_idx + 1].y - hull[hi_idx].y), cnd) >= 0:
        hi_idx += 1
    ans[q_id] = dot(hull[hi_idx], cnd)
print(ans)
