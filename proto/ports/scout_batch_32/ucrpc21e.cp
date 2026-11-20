// UCRPC '21 E (DMOJ: ucrpc21e) - best of two scoring streaks
// Ideal v2 port. original: corpus/submissions/3944571.cpp
//
// Two teams each have scoring intervals [l, r] worth 1 point per second
// (3 for the bonus kind). At every second only the team currently scoring
// faster counts. Print the total: an event sweep over interval endpoints.
enum Team = Home | Away
let (len, n, m): (1..=1e9, 1..=1e5, 1..=1e5) = read()

type Event = (t: Int, team: Team, d: Int)
var evs: List<Event> = []
for (team, cnt) in [(Home, n), (Away, m)]:
    for _ in 0..cnt:
        let (l, r, bonus): (1..=len, l..=len, Bool) = read()
        let pts = if bonus: 3 else: 1
        evs.push((l, team, pts))
        evs.push((r + 1, team, -pts))
sort(mut evs, by: e => e.t)

var rate: [Team]Int = 0
var (total, last) = (0, 0)
for (t, team, d) in evs:
    total += (t - last) * max(rate)
    rate[team] += d
    last = t
print(total)
