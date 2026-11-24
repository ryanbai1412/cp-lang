// Codeforces 2229G: Roadworks, tuned.
// port.cp's sweep, taking the next record by a two-way pointer step instead of building the
// merge_sorted list of (value, side, index) up front, and scanning the other side's met records
// with one pointer instead of first_true then a loop. As in orig.cpp, the record lists are
// reserved at n, fields and input are stored at their bounds, and paths are walked, not collected.

enum Side: Left | Right
fn flip(s: Side) -> Side = match s:
    Left => Side.Right
    Right => Side.Left

struct House(x: 0..=200_000, v: 1..=10 ** 9, d: 0..=1_000_200_000, pref: Int)

@!multitest
let (n, k): (1..=200_000, 1..=1_000_000_000) = read()
domain Pos = 1..=n
let x: Pos = read()
let v: [Pos]1..=10 ** 9 = read()
let road_d: [0..n - 1]0..=10 ** 9 = read()

fn records<Path>(path: Path) -> List<House>:
    var out: List<House> = []
    out.reserve(n)
    var (pref, d) = (0, 0)
    var prev: ?Pos = None
    for i in path:
        let s = abs(i - x)
        pref += v[i]
        if let p = prev: d = max(d + 1, road_d[min(i.ord, p.ord)])
        prev = i
        if out.is_empty or out.last!.v <= v[i]: out.push(House(s, v[i], d, pref))
    out

let side: [Side]List<House> = [Side] s => match s:
    Left => records((Pos.first!..=x).rev)
    Right => records(x..=Pos.last!)
var val: [Side]List<Int> = [Side] _ => [0]
var crossed: [Side]Int = 0
var ans = v[x] * k
var next: [Side]Int = 1
fn waiting(z: Side) -> Bool = next[z] < side[z].len
while waiting(Side.Left) or waiting(Side.Right):
    let right_first = waiting(Side.Right) and (
        not waiting(Side.Left) or
            side[Side.Right][next[Side.Right]].v < side[Side.Left][next[Side.Left]].v
    )
    let z = if right_first: Side.Right else: Side.Left
    let i = next[z]
    next[z] += 1
    let (o, h) = (side[z][i - 1], side[z][i])
    var best = val[z][i - 1] + o.v * (h.d - (h.x - o.x)) + (h.pref - o.pref)
    let opp = flip(z)
    let met = val[opp].len
    var j = crossed[opp]
    while j < met and side[opp][j].v <= h.v:
        let op = side[opp][j]
        if op.d + h.x + op.x <= h.d:
            best max= val[opp][j] + op.v * (h.d - (h.x + op.x + 1)) + op.pref + h.pref - v[x]
        j += 1
    crossed[opp] = j
    val[z].push(best - h.d * h.v)
    if h.d <= k: ans max= val[z][i] + h.v * k
print(ans)
