// Runnable port (proto/cp.py lib-run) of mccc4j4.cp
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
    groups[g].sort(by: t => (-pts[t], t.to_idx))
    out.push(groups[g][k - 1].to_idx + 1)

var line = ""
for i in 0..out.len:
    if i > 0: line = line + " "
    line += "{out[i]}"
print(line)
