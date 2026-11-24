// Source: Mock CCC '22 Contest 2 J4, Hockey Bracket, https://dmoj.ca/problem/mccc4j4
// limits: 1000 ms, 256 MB
// Problem: N <= 2000 countries in N / S groups of size S play every pair in their group once
//   (win 3, tie 1 each). Given the groups, the (S - 1) N / 2 results (W, L, T for the first)
//   and K, print the K-th placed country of each group (points down, then lower number first).
// Solution: add up the points, sort each group by (-points, number) and take entry K.
//   O(N log N + matches).
// original: corpus/submissions/4264934.cpp

let (n, m): (Int, Int) = read()
domain Team = 1..=n
domain Group = 0..(n / m)

var groups: [Group]List<Team> = [Group] _ => []
for g in Group:
    for _ in 0..m:
        let t: Team = read()
        groups[g].push(t)

var pts: [Team]Int = [Team] _ => 0
for _ in 0..((m - 1) * n / 2):
    let (u, v, res): (Team, Team, Char) = read()
    if res == 'W':
        pts[u] += 3
    elif res == 'T':
        pts[u] += 1
        pts[v] += 1
    else:
        pts[v] += 3

let k: Int = read()
var out: List<Int> = []
for g in Group:
    groups[g].sort(by: t => (-pts[t], t.ord))
    out.push(groups[g][k - 1].ord + 1)

var line = ""
for i in 0..out.len:
    if i > 0: line += " "
    line += "{out[i]}"
print(line)
