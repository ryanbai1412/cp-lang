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

// Partition of preorder positions [0, n) by last-painted timestamp.
struct PaintMap:
    segs: Map<Int, (r: Int, t: Time)>

    fn split(mut self, x: Int):
        if let (sl, val) = self.segs.last_le(x):
            if sl < x and val.r > x:
                self.segs[sl] = (r: x, t: val.t)
                self.segs[x] = (r: val.r, t: val.t)

    fn assign(mut self, l: Int, r: Int, t_now: Time, mut out: List<(t: Time, w: Int)>):
        self.split(l)
        self.split(r)
        var pos = l
        while pos < r:
            let seg = self.segs.remove(pos)!
            out.push((t: seg.t, w: seg.r - pos))
            pos = seg.r
        self.segs[l] = (r: r, t: t_now)

var init_time: [0..n]Time = Time(1)
for v in init_slime: init_time[tree.pos(v).ord] = Time(2)

var init_segs = Map<Int, (r: Int, t: Time)>()
var run_l = 0
for i in 1..n:
    if init_time[i] != init_time[run_l]:
        init_segs[run_l] = (r: i, t: init_time[run_l])
        run_l = i
init_segs[run_l] = (r: n, t: init_time[run_l])
var paint = PaintMap(init_segs)

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
        paint.assign(range.lo.ord, range.hi.ord, t_now, mut chunks)
    cur = c

    chunks.sort(by: x => desc(x.t))
    var matched = 0
    for chunk in chunks:
        var need = chunk.w
        var it: ?Time = chunk.t
        while let u = it, need > 0:
            let (prv, cnt) = supply.get(u)
            let take = min(cnt, need)
            matched += take
            need -= take
            supply.set(u, (prv: prv, cnt: cnt - take))
            if take == cnt:
                if let p = prv:
                    supply.unite(u, p)
                else:
                    it = none
        tot_ans += need

    supply.set(t_now, (prv: t_now - 1, cnt: matched))
    ans.push(tot_ans)

print(ans)
