// Codeforces 2237 H: Slime and Queries.
// Each query paints the path cur -> c with timestamp t_now. Every vertex on
// the path whose last paint was t_old demands 1 unit of slime from the newest
// non-empty supply bucket <= t_old; unmatched demand increases the answer, and
// all matched slime moves into bucket t_now.
@!multitest
use std.dsu.{DsuOf}

let (n, m, q): (2..=100_000, 2..=100_000, 1..=100_000) = read()
domain V = 1..=n
domain Time = 1..=q + 2
let edges: [0..n - 1](u: V, v: V) = read()
let tree = Rooted(Graph.undirected(edges), V.first)
let init_slime: [0..m]V = read()

// Partition of preorder positions by last-painted timestamp.
var init_time: [tree.Pre]Time = Time(1)
for v in init_slime: init_time[tree.pos(v)] = Time(2)
var paint = IntervalMap.from(init_time)

var supply = DsuOf<Time, (prv: ?Time, cnt: Int)>(_ => (prv: none, cnt: 0), (_, b) => b)
supply.set(Time(2), (prv: none, cnt: m))

var cur = init_slime[m - 1]
var tot_ans = 0
var ans = List<Int>()
for t_now in Time(3)..:
    let raw_c: V = read()
    let c = raw_c.cyc(tot_ans)

    var chunks = List<(t: Time, w: Int)>()
    for (range, _) in tree.chains(cur, c):
        paint.assign(range, t_now, on_old: (r, t) => chunks.push((t:, w: r.len)))
    cur = c

    chunks.sort(by: x => -x.t.to_idx)
    var matched = 0
    for chunk in chunks:
        var need = chunk.w
        var it: ?Time = chunk.t
        while let u = it, need > 0:
            let (prv, cnt) = supply.get(u)
            let take = min(cnt, need)
            matched += take
            need -= take
            supply.set(u, (prv:, cnt: cnt - take))
            if take == cnt:
                if let p = prv:
                    supply.unite(u, p)
                else:
                    it = none
        tot_ans += need

    supply.set(t_now, (prv: t_now - 1, cnt: matched))
    ans.push(tot_ans)

print(ans)
