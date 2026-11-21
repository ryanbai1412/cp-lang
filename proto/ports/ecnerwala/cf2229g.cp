// Codeforces 2229 G: compute earliest arrival times left and right from X,
// keep only prefix-record houses on each side, and merge them by increasing V.
struct House(x: Int, v: Int, d: Int, pref: Int, val: Int)

@!multitest
let (n, k, x_raw): (Int, Int, Int) = read()
let x = x_raw - 1
let v: [0..n]Int = read()
let road_d: [0..n - 1]Int = read()

var d: [0..n]Int = 0
for i in x + 1..n: d[i] = max(d[i - 1] + 1, road_d[i - 1])
for i in (0..x).rev: d[i] = max(d[i + 1] + 1, road_d[i])

fn collect(ord: List<Int>) -> List<House>:
    var out: List<House> = []
    var pref = 0
    for i in ord:
        pref += v[i]
        if out.is_empty or out.last!.v <= v[i]:
            out.push(House(abs(i - x), v[i], d[i], pref, 0))
    out

var side: [0..2]List<House> = [collect(List((0..=x).rev)), collect(List(x..n))]

var ans = v[x] * k
var proc_it: [0..2]Int = 1
var ins_it: [0..2]Int = 0
while proc_it[0] < side[0].len or proc_it[1] < side[1].len:
    let z = Int(
        proc_it[1] < side[1].len and (
            proc_it[0] == side[0].len or side[1][proc_it[1]].v < side[0][proc_it[0]].v
        )
    )
    let i = proc_it[z]
    proc_it[z] += 1
    let o = side[z][i - 1]
    let h = side[z][i]
    var val = o.val + o.v * (h.d - (h.x - o.x)) + (h.pref - o.pref)
    let oz = 1 - z
    while ins_it[oz] < proc_it[oz] and side[oz][ins_it[oz]].v <= h.v:
        let op = side[oz][ins_it[oz]]
        ins_it[oz] += 1
        if op.d + h.x + op.x <= h.d:
            val max= op.val + op.v * (h.d - (h.x + op.x + 1)) + op.pref + h.pref - v[x]
    val -= h.d * h.v
    side[z][i].val = val
    if h.d <= k: ans max= val + h.v * k
print(ans)
