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

// Supply buckets ordered by timestamp; empty buckets redirect to their predecessor.
var supply = DsuOf<Time, (prv: ?Time, cnt: Int)>(_ => (none, 0), (_, b) => b)
supply.set(Time(2), (none, m))

var cur = init.last
var tot_ans = 0
var ans: List<Int> = []

for t_now in Time(3)..:
    let c = read<V>().cyc(tot_ans)
    var chunks = [
        (t, w: r.len)
        for (range, _) in tree.chains(cur, c)
        for (r, t) in paint.assign(range, t_now)
    ]
    cur = c

    chunks.sort(by: x => desc(x.t))
    var matched = 0
    for (t, mut need) in chunks:
        var it: ?Time = t
        while let u = it, need > 0:
            let (prv, cnt) = supply.get(u)
            let take = min(cnt, need)
            matched += take
            need -= take
            supply.set(u, (prv, cnt - take))
            if take == cnt:
                if let p = prv: supply.unite(u, p) else: it = none
        tot_ans += need

    supply.set(t_now, (t_now - 1, matched))
    ans.push(tot_ans)

print(ans)
