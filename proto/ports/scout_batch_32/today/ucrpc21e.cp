// Runnable port (proto/cp.py lib-run) of statement-first ucrpc21e.cp
// original: corpus/submissions/3944571.cpp
let (_len, n, m): (Int, Int, Int) = read()

// One event per interval endpoint: at time t, team `who` gains `d` points/sec.
var evs: List<(t: Int, d: Int, who: Int)> = []
fn read_team(cnt: Int, who: Int):
    for _ in 0..cnt:
        let (l, r, kind): (Int, Int, Int) = read()
        let pts = if kind != 0: 3 else: 1
        evs.push((t: l, d: pts, who: who))
        evs.push((t: r + 1, d: -pts, who: who))
read_team(n, 0)
read_team(m, 1)
sort(mut evs)

var rate = [0, 0]
var (ans, last_t) = (0, 0)
for e in evs:
    ans += (e.t - last_t) * max(rate[0], rate[1])
    rate[e.who] += e.d
    last_t = e.t
print(ans)
