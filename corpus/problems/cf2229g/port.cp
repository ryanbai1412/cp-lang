// Source: Codeforces 2229G (Roadworks), https://codeforces.com/contest/2229/problem/G
// Problem: n <= 2e5 houses in a row with h_i <= 1e9; road i (between i and i + 1) is built on day
//   d_i. Start at house x; on each day 1..k (k <= 1e9) roads due open, you may step to a
//   neighbour over a built road, then gain h of your house. Print the maximum total. Multitest,
//   sum n <= 2e5.
// Solution: d is the earliest arrival day along each side of x, and it only pays to settle at a
//   house worth more than any passed on its side, so each side keeps its prefix records by value.
//   val[r] makes v * t + val the best total by day t >= d staying at record r. Records are met in
//   increasing value across both sides (merge_sorted); r is reached from the previous record on
//   its side or from an already-met record on the other side through x (whose value both prefix
//   sums count, hence - v[x]); a pointer per side visits each candidate once: O(n).
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

fn records(path: List<Pos>) -> List<House>:
    var out: List<House> = []
    var (pref, d) = (0, 0)
    for (s, i) in path.items:
        pref += v[i]
        if s > 0: d = max(d + 1, road_d[min(i.ord, path[s - 1].ord)])
        if out.is_empty or out.last!.v <= v[i]: out.push(House(s, v[i], d, pref))
    out

let side: [Side]List<House> = [Side] s => match s:
    Left => records([p for p in (Pos.first!..=x).rev])
    Right => records([p for p in x..=Pos.last!])
let order = merge_sorted(
    [(h.v, Side.Left, i) for (i, h) in side[Side.Left].items if i > 0],
    [(h.v, Side.Right, i) for (i, h) in side[Side.Right].items if i > 0],
)

var val: [Side]List<Int> = [Side] _ => [0]
var crossed: [Side]Int = 0
var ans = v[x] * k
for (_, z, i) in order:
    let (o, h) = (side[z][i - 1], side[z][i])
    var best = val[z][i - 1] + o.v * (h.d - (h.x - o.x)) + (h.pref - o.pref)
    let opp = flip(z)
    let met = val[opp].len
    let upto = (crossed[opp]..met).first_true(j => side[opp][j].v > h.v) ?? met
    for j in crossed[opp]..upto:
        let op = side[opp][j]
        if op.d + h.x + op.x <= h.d:
            best max= val[opp][j] + op.v * (h.d - (h.x + op.x + 1)) + op.pref + h.pref - v[x]
    crossed[opp] = upto
    val[z].push(best - h.d * h.v)
    if h.d <= k: ans max= val[z][i] + h.v * k
print(ans)
