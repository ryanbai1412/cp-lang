// Source: IOI 2015 Day 1 P1 (Boxes with Souvenirs), https://dmoj.ca/problem/ioi15p1
// Problem: n teams (n <= 1e7) sit at sorted positions on a circle of l sections (l <= 1e9);
//   start at section 0, carry at most k souvenirs per trip, give one to every team and
//   finish back at 0. Print the least total walking distance (sections moved).
// Solution: some optimal plan serves the teams by position: a clockwise prefix and an
//   anticlockwise suffix by round trips, each taking the k farthest teams it has left, and
//   at most one full lap (cost l) for k consecutive teams between them. cw[i] (first i
//   teams clockwise) and ccw[i] (teams i.. anticlockwise) are O(1) recurrences; take the
//   best split i, with or without a lap over the k teams after it. O(n).
// original: corpus/submissions/5723345.cpp (grader function delivery)
// stdin here, as the IOI sample grader: n k l, then the n positions in order.
let n: 1..=10_000_000 = read()
let k: 1..=n = read()
let l: 1..=1_000_000_000 = read()
domain Team = 0..n
let pos: [Team]0..l = read()

// cw[c]: the teams before cut c served by clockwise round trips, the k farthest a trip;
// ccw[c]: the teams after cut c, anticlockwise
var cw: [Team.cuts]Int = 0
for t in Team: cw[t.r] = 2 * pos[t] + cw[Team.cuts.at(Int(t.r) - k) ?? Team.cuts.first!]
var ccw: [Team.cuts]Int = 0
for t in Team.rev: ccw[t.l] = 2 * (l - pos[t]) + ccw[Team.cuts.at(Int(t.l) + k) ?? Team.cuts.last!]

// split the teams at cut c, with at most one full lap taking the k teams after it
let lap = (c: Team.cuts) => l + ccw[Team.cuts.at(Int(c) + k) ?? Team.cuts.last!]
print(Team.cuts.map(c => cw[c] + min(ccw[c], lap(c))).min()!)
