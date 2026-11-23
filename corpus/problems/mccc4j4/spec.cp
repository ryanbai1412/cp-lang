// mccc4j4: teams play round robins in groups of m; a win is 3 points, a tie 1. Sort each group by
// points, then id, and print each group's k-th team.
let (n, m) = read<(1..=2_000, 1..=2_000)>()
domain Team = 1..=n
domain Group = 0..(n / m)
enum Outcome:
    @tag("W") Win
    @tag("T") Tie
    @tag("L") Loss
let groups = read<[Group, m]Team>()
var pts: [Team]Int = 0
for g in read<[(m - 1) * n / 2](u: Team, v: Team, res: Outcome)>():
    match g.res:
        Win => pts[g.u] += 3
        Loss => pts[g.v] += 3
        Tie =>
            pts[g.u] += 1
            pts[g.v] += 1
let k = read<1..=m>()
print(Group.map(g => groups[g].sorted(by: t => (desc(pts[t]), t))[k - 1]))
