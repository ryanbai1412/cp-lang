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
enum Side: Left | Right
fn flip(s: Side) -> Side = match s:
    Left => Side.Right
    Right => Side.Left

struct House(x: Int, v: Int, d: Int, pref: Int)

@!multitest
let (n, k): (1..=200_000, 1..=1_000_000_000) = read()
domain Pos = 1..=n
let x: Pos = read()
let v: [Pos]Int = read()
let road_d: [0..n - 1]Int = read()

// The houses on a walk from x that are at least as valuable as every house before them.
fn records(path: List<Pos>) -> List<House>:
    var (d, pref) = (0, 0)
    var all: List<House> = []
    for (s, i) in path.items:
        if s > 0: d = max(d + 1, road_d[min(i.ord, path[s - 1].ord)])
        pref += v[i]
        all.push(House(s, v[i], d, pref))
    [all[s] for s in all.record_highs(by: h => (h.v, h.x))]

let side: [Side]List<House> = [Side] s => match s:
    Left => records((Pos.first!..=x).rev.collect())
    Right => records((x..=Pos.last!).collect())
let order = merge_sorted(
    [(Side.Left, i) for i in 1..side[Side.Left].len],
    [(Side.Right, i) for i in 1..side[Side.Right].len],
    by: t => side[t.0][t.1].v,
)

var val: [Side]List<Int> = [Side] _ => [0]
var crossed: [Side]Int = 0
var ans = v[x] * k
for (z, i) in order:
    let (o, h) = (side[z][i - 1], side[z][i])
    let stay = val[z][i - 1] + o.v * (h.d - (h.x - o.x)) + (h.pref - o.pref)
    let opp = flip(z)
    let met = val[opp].len
    let upto = (crossed[opp]..met).first_true(j => side[opp][j].v > h.v) ?? met
    let cross = max(
        val[opp][j] + op.v * (h.d - (h.x + op.x + 1)) + op.pref + h.pref - v[x]
        for (j, op) in side[opp][crossed[opp]..upto].items if op.d + h.x + op.x <= h.d
    )
    crossed[opp] = upto
    val[z].push(max(stay, cross ?? stay) - h.d * h.v)
    if h.d <= k: ans max= val[z][i] + h.v * k
print(ans)
