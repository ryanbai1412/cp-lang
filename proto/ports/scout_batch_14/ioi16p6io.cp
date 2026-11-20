// IOI 2016 Aliens: cover n points in an m x m grid with at most k diagonal
// squares of minimum total area. WQS (Aliens) binary search + Li Chao tree DP.
// original: corpus/submissions/4730558.cpp
use local.ds.li_chao.{LiChao}                                     // WISH P8, P12

let n: 1..=100000 = read()
let m: 1..=1000000 = read()
let k: 1..=100000 = read()
let raw: [n](r: 0..=1000000, c: 0..=1000000) = read()

// Normalize points into intervals [min(r, c), max(r, c)] and strip dominated ones.
type Seg = (l: Int, r: Int)
var norm: List<Seg> = []                                          // WISH P1
for p in raw: norm.push((min(p.r, p.c), max(p.r, p.c)))
var segs: List<Seg> = []
var max_r = -1
for s in sorted(norm, by: s => (s.l, -s.r)):
    if s.r > max_r:
        max_r = s.r
        segs.push(s)

struct CountLine(k: Int, b: Int, used: Int):                      // WISH P8
    fn at(self, x: Int) -> (cost: Int, used: Int) = (k * x + b, used)

fn eval_penalty(lam: Int) -> (cost: Int, used: Int):
    var hull = LiChao<CountLine, (cost: Int, used: Int)>.new(0, m)  // WISH P12
    let last_r = segs[segs.len - 1].r + 1
    hull.add(CountLine(-2 * last_r, last_r * last_r, 0))
    var (best_cost, best_used) = (0, 0)                           // WISH P2
    for i in (0..segs.len).rev:
        let q = hull.min_at(segs[i].l)!
        var cost = q.cost + lam + segs[i].l * segs[i].l
        let used = q.used + 1
        if i > 0 and segs[i].l <= segs[i - 1].r:
            let ov = segs[i - 1].r - segs[i].l + 1
            cost -= ov * ov
        (best_cost, best_used) = (cost, used)
        if i > 0:
            let r = segs[i - 1].r + 1
            hull.add(CountLine(-2 * r, cost + r * r, used))
    (best_cost, best_used)

let lam = (0..=m * m).first_where(pen => eval_penalty(pen).used <= k) ?? (m * m)
let res = eval_penalty(lam)
print(res.cost - k * lam)
