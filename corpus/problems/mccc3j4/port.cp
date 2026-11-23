// Source: Mock CCC '22 Contest 1 J4 (Snowball Fight), https://dmoj.ca/problem/mccc3j4
// Problem: N <= 2000 students each start with one target in a queue. In each of T <= 2000
//   rounds, every student with a nonempty queue throws at its front and pops it; then each
//   hit student appends its throwers in increasing order. Print each student's last target.
// Solution: simulate: a list per student with a head index as its queue; collect the round's
//   (target, thrower) pairs in thrower order before appending any. Each round appends at most
//   N entries: O(N T).
// original: corpus/submissions/4223440.cpp

let (n, t): (Int, Int) = read()
domain Player = 1..=n
let init: [Player]Player = read()

var q: [Player]List<Player> = [Player] p => [init[p]]
var head: [Player]Int = [Player] _ => 0
var last: [Player]?Player = [Player] _ => None

for _ in 0..t:
    var arrivals: List<(to: Player, from: Player)> = []
    for p in Player:
        if head[p] < q[p].len:
            let target = q[p][head[p]]
            head[p] += 1
            last[p] = target
            arrivals.push((to: target, from: p))
    for item in arrivals:
        q[item.to].push(item.from)

var line = ""
for p in Player:
    if p.to_idx > 0: line += " "
    line += "{last[p]!}"
print(line)
