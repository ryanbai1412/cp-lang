@!multitest
let (n, m, q): (2..=100_000, 2..=100_000, 1..=100_000) = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let tree = Rooted(Graph.undirected(edges), V.first)

let init_slime: [0..m]V = read()
var cur = init_slime[m - 1]

domain Time = 1..=q + 2

var init_time: [0..n]Time = Time(1)
for v in init_slime:
    init_time[tree.pos(v).ord] = Time(2)

var segs = Map<Int, (r: Int, t: Time)>()
var run_l = 0
for i in 1..n:
    if init_time[i] != init_time[run_l]:
        segs[run_l] = (r: i, t: init_time[run_l])
        run_l = i
segs[run_l] = (r: n, t: init_time[run_l])

var dsu = Dsu<Time>()
var prv: [Time]?Time = none
var cnt: [Time]Int = 0
cnt[Time(2)] = m

fn split(mut segs: Map<Int, (r: Int, t: Time)>, x: Int):
    if let (sl, val) = segs.last_le(x):
        if sl < x and val.r > x:
            segs[sl] = (r: x, t: val.t)
            segs[x] = (r: val.r, t: val.t)

var tot_ans = 0
var ans = List<Int>()
for t_now in Time(3)..:
    let raw_c: V = read()
    let c = raw_c.cyc(tot_ans)

    var chunks = List<(t: Time, w: Int)>()
    for (range, _) in tree.chains(cur, c):
        let (l, r) = (range.lo.ord, range.hi.ord)
        split(mut segs, l)
        split(mut segs, r)
        var pos = l
        while pos < r:
            let seg = segs.remove(pos)!
            chunks.push((t: seg.t, w: seg.r - pos))
            pos = seg.r
        segs[l] = (r: r, t: t_now)
    cur = c

    sort(mut chunks, by: x => desc(x.t))
    var cur_cnt = 0
    var cur_ans = 0
    for chunk in chunks:
        var w = chunk.w
        var it: ?Time = dsu.find(chunk.t)
        while let u = it, w > 0:
            let take = min(cnt[u], w)
            cur_cnt += take
            cnt[u] -= take
            w -= take
            if cnt[u] == 0:
                if let p = prv[u]:
                    let b = dsu.find(p)
                    let (np, nc) = (prv[b], cnt[b])
                    let (kept, _) = dsu.merge(u, b)!
                    prv[kept] = np
                    cnt[kept] = nc
                    it = kept
                else:
                    it = none
        cur_ans += w

    cnt[t_now] = cur_cnt
    prv[t_now] = t_now - 1
    tot_ans += cur_ans
    ans.push(tot_ans)

print(ans)
