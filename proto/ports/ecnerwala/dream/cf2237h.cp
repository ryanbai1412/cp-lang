// Codeforces 2237 H.
// Each query paints the path `cur -> c` with timestamp `t_now`. Every vertex on
// the path whose last paint was `t_old` demands 1 unit of slime from the newest
// non-empty supply bucket `<= t_old`; unmatched demand increases `missing`, and
// all matched slime moves into bucket `t_now`.

@!multitest
let (n, m, q): (2..=100_000, 2..=100_000, 1..=100_000) = read()
domain V = 1..=n
domain Slime = 0..m
domain Time = 1..=q + 2
let never = Time.first
let start = (Time.first + 1)!
let tree = Rooted(Graph.undirected(read<[n - 1](u: V, v: V)>()), V.first)
let init: [Slime]V = read()

// Partition of preorder positions by last-painted timestamp.
var paint = IntervalMap<tree.Pre, Time>(never)
for v in init: paint.assign(tree.pos(v)..=tree.pos(v), start)

// Slime waiting to move, keyed by the time it was last moved; only non-empty buckets.
var supply = Map<Time, Int>()
supply[start] = m

// Matches up to `need` units from the newest buckets at or before `t`.
// Each emptied bucket leaves `supply`, so all calls cost O((q + m) log q) total.
fn take(need: Int, before t: Time) -> Int:
    var got = 0
    while got < need, let (u, cnt) = supply.last_le(t):
        let k = min(cnt, need - got)
        got += k
        if k == cnt: supply.remove(u) else: supply[u] -= k
    got

var cur = init[Slime.last]
var missing = 0
var ans: List<Int> = []

for t_now in (Time.first + 2)!..=Time.last:
    let c = read<V>().cyc(missing)
    let chunks = [
        (
            t,
            need: r.len,
        ) for (range, _) in tree.chains(cur, c) for (r, t) in paint.assign(range, t_now)
    ].sort(by: x => desc(x.t))
    cur = c

    let demand = chunks.map(c => c.need).sum()
    let moved = chunks.map(c => take(c.need, before: c.t)).sum()
    missing += demand - moved
    if moved > 0: supply[t_now] = moved
    ans.push(missing)

print(ans)
