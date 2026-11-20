// Mock CCC '22 Contest 2 J4 — Snowball Fight / Group Stage Ranking (CP Lang v2 spec)
// original: corpus/submissions/4264934.cpp (note: original pastes a 23-line unused struct DSU)
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Partition n teams into n/m groups of size m, tally round-robin W/T/L match points,
// sort each group by descending points then ascending team ID, and print each group's k-th team.

let (n, m): (1..=2000, 1..=n) = read()
assume n % m == 0

domain Team = 1..=n
domain Group = 0..(n / m)
domain Slot = 1..=m

var groups: [Group, Slot]Team = read()

@tags('W', 'T', 'L')
enum Outcome = Win | Tie | Loss

var pts: [Team]Int = 0
for _ in 0..((m - 1) * n / 2):
    let (u, v, res): (Team, Team, Outcome) = read()
    match res:
        Win  => pts[u] += 3
        Tie  =>
            pts[u] += 1
            pts[v] += 1
        Loss => pts[v] += 3

let k: Slot = read()
// FRICTION MC6: `sort(mut groups.row(g), by: ...)` assumes `g.row(r)` yields a
// mutable slice compatible with `sort(mut ...)`, and mixing `desc(pts[t])` with
// ascending `t` in a single key tuple relies on `Desc<T>` boxing inside tuples.
for g in Group:
    sort(mut groups.row(g), by: t => (desc(pts[t]), t))

print([groups[g, k] for g in Group], sep: " ")
