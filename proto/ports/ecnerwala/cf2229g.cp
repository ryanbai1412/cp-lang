// Codeforces 2229 G.
//
// Start at house X at time 0 and walk left or right; road i opens at
// road_d[i], so `d` is the earliest arrival along each side. It only pays to
// move on to a house worth more, so each side keeps its prefix records by
// value, met in order of increasing v across both sides. A record's `val` makes
// v * t + val the best total by time t >= d while staying at it. A record is
// reached from the previous record on its own side, or from an already-met
// record on the other side by walking back through X (whose value both prefix
// sums count once each, hence `- v[x]`); either way every house passed is
// collected once.
use std.seq.{merge_sorted}

struct House(x: Int, v: Int, d: Int, pref: Int)

@!multitest
let (n, k, x_in): (Int, Int, Int) = read()
let x = x_in - 1
let v: [0..n]Int = read()
let road_d: [0..n - 1]Int = read()

fn records(path: List<Int>) -> List<House>:
    var out: List<House> = []
    var (pref, d) = (0, 0)
    for (s, i) in path.items:
        pref += v[i]
        if s > 0: d = max(d + 1, road_d[min(i, path[s - 1])])
        if out.is_empty or out.last!.v <= v[i]: out.push(House(s, v[i], d, pref))
    out

let side: [0..2]List<House> = [records(List((0..=x).rev)), records(List(x..n))]
let order = merge_sorted(
    [(h.v, 0, i) for (i, h) in side[0].items if i > 0],
    [(h.v, 1, i) for (i, h) in side[1].items if i > 0],
)

var val: [0..2]List<Int> = [0]
var crossed: [0..2]Int = 0
var ans = v[x] * k
for (_, z, i) in order:
    let (o, h) = (side[z][i - 1], side[z][i])
    var best = val[z][i - 1] + o.v * (h.d - (h.x - o.x)) + (h.pref - o.pref)
    let other = 1 - z
    let met = val[other].len
    let upto = bsearch_first(crossed[other]..met, j => side[other][j].v > h.v) ?? met
    for j in crossed[other]..upto:
        let op = side[other][j]
        if op.d + h.x + op.x <= h.d:
            best max= val[other][j] + op.v * (h.d - (h.x + op.x + 1)) + op.pref + h.pref - v[x]
    crossed[other] = upto
    val[z].push(best - h.d * h.v)
    if h.d <= k: ans max= val[z][i] + h.v * k
print(ans)
