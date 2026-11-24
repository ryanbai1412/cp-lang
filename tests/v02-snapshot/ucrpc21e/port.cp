// Source: UCRPC F21 E (Feeding Friendsy), https://dmoj.ca/problem/ucrpc21e
// Problem: a game of T time units; the upper and the lower mouth each open on n (resp. m)
//   disjoint sorted intervals [a, b], each worth 1 or (rainbow) 3 points per goal. Each time
//   unit you throw at most one ball into one open mouth. Print the most points possible.
// Solution: each time unit independently takes the better of the two mouths, so the answer
//   is the sum over time of max(upper rate, lower rate). One event per interval endpoint
//   (+pts at a, -pts at b + 1), sorted; the sweep adds gap * max rate. O((n + m) log).
// original: corpus/submissions/3944571.cpp
let (_len, n, m): (Int, Int, Int) = read()

// One event per interval endpoint: at time t, team `who` gains `d` points/sec.
var evs: List<(t: Int, d: Int, who: Int)> = []
fn read_team(cnt: Int, who: Int):
    for _ in 0..cnt:
        let (l, r, kind): (Int, Int, Int) = read()
        let pts = if kind != 0: 3 else: 1
        evs.push((t: l, d: pts, who:))
        evs.push((t: r + 1, d: -pts, who:))
read_team(n, 0)
read_team(m, 1)
evs.sort()

var rate = [0, 0]
var (ans, last_t) = (0, 0)
for e in evs:
    ans += (e.t - last_t) * max(rate[0], rate[1])
    rate[e.who] += e.d
    last_t = e.t
print(ans)
