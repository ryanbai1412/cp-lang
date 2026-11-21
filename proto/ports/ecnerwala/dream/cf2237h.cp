// Each query paints the path cur -> c with timestamp t_now. Every vertex on
// the path whose last paint was t_old demands 1 unit of slime from the newest
// non-empty supply bucket <= t_old; unmatched demand increases the answer, and
// all matched slime moves into bucket t_now.
@!multitest
let (n, m, q): (2..=1e5, 2..=1e5, 1..=1e5) = read()
domain V = 1..=n
domain Time = 1..=q + 2
let tree = Rooted(Graph.undirected(read<[n - 1](u: V, v: V)>()), V.first)
let init: [0..m]V = read()

// Partition of preorder positions by last-painted timestamp.
var paint = IntervalMap<tree.Pre, Time>(Time(1))
for v in init: paint.assign(tree.pos(v)..=tree.pos(v), Time(2))

// Slime waiting to move, keyed by the time it was last moved; only non-empty buckets.
var supply = Map<Time, Int>()
supply[Time(2)] = m

// Moves up to `need` units out of the newest buckets at or before `t`; returns how many.
// Each emptied bucket leaves the map, so all the takes cost O((q + m) log q) together.
fn take(need: Int, before t: Time) -> Int:
    var got = 0
    while got < need, let (u, cnt) = supply.last_le(t):
        let k = min(cnt, need - got)
        got += k
        if k == cnt: supply.remove(u) else: supply[u] -= k
    got

var cur = init.last
var missing = 0
var ans: List<Int> = []

for t_now in Time(3)..:
    let c = read<V>().cyc(missing)
    let chunks = [
        (t, need: r.len)
        for (range, _) in tree.chains(cur, c)
        for (r, t) in paint.assign(range, t_now)
    ].sort(by: x => desc(x.t))
    cur = c

    var moved = 0
    for (t, need) in chunks:
        let got = take(need, before: t)
        moved += got
        missing += need - got
    if moved > 0: supply[t_now] = moved
    ans.push(missing)

print(ans)
