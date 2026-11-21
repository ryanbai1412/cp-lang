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
struct House(x: Int, v: Int, d: Int, pref: Int)

@!multitest
let (n, k): (1..=2e5, 1..=1e9) = read()
domain Pos = 1..=n
let x: Pos = read()
let v: [Pos]Int = read()
let road_d: [Pos.cuts.inner]Int = read()

fn records(path: List<Pos>) -> List<House>:
    let d = path.windows(2).scan(0, (t, w) => max(t + 1, road_d[cut(w[0], w[1])]))
    let pref = path.scan(0, (s, i) => s + v[i])
    [House(s, v[i], d[s], pref[s]) for (s, i) in path.items].prefix_records(by: h => h.v)

let side: [Side]List<House> = [Side] s => records(Pos.walk(from: x, s))

var val: [Side]List<Int> = [Side] _ => [0]
var crossed: [Side]Int = 0
var ans = v[x] * k
for (z, i) in merge(side.map(s => s.indices.skip(1)), by: (z, i) => side[z][i].v):
    let (o, h) = (side[z][i - 1], side[z][i])
    let stay = val[z][i - 1] + o.v * (h.d - (h.x - o.x)) + (h.pref - o.pref)
    let opp = z.flip
    let pending = side[opp].items[crossed[opp]..val[opp].len]
    let fresh = pending.take_while(op => op.v <= h.v)
    let cross = max(
        val[opp][j] + op.v * (h.d - (h.x + op.x + 1)) + op.pref + h.pref - v[x]
        for (j, op) in fresh if op.d + h.x + op.x <= h.d
    )
    crossed[opp] += fresh.len
    val[z].push(max(stay, cross ?? stay) - h.d * h.v)
    if h.d <= k: ans max= val[z][i] + h.v * k
print(ans)
