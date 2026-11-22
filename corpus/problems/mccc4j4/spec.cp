// mccc4j4: teams play round robins in groups of m; a win is 3 points, a tie 1. Sort each group by
// points, then id, and print each group's k-th team.
let (n, m) = read<(1..=2_000, 1..=2_000)>()
domain Team = 1..=n
enum Outcome:
    @tag("W") Win
    @tag("T") Tie
    @tag("L") Loss
var groups = read<[n / m, m]Team>()
var pts: [Team]Int = 0
for g in read<[(m - 1) * n / 2](u: Team, v: Team, res: Outcome)>():
    if g.res == Outcome.Win:
        pts[g.u] += 3
    elif g.res == Outcome.Loss:
        pts[g.v] += 3
    else:
        pts[g.u] += 1
        pts[g.v] += 1
let k = read<1..=m>()
for g in groups.keys.0:
    groups[g].sort(by: t => (desc(pts[t]), t))
print(groups.keys.0.map(g => groups[g, k - 1]))
