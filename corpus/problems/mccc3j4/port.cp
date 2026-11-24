// Source: Mock CCC '22 Contest 1 J4 (Snowball Fight), https://dmoj.ca/problem/mccc3j4
// Problem: N <= 2000 students each start with one target in a queue. In each of T <= 2000
//   rounds, every student with a nonempty queue throws at its front and pops it; then each
//   hit student appends its throwers in increasing order. Print each student's last target.
// Solution: simulate with a deque per student; collect the round's (target, thrower) pairs in
//   thrower order before appending any. Round one throws at every first target, so the last
//   target starts as it. Each round appends at most N entries: O(N T).
// original: corpus/submissions/4223440.cpp

let (n, t): (1..=2000, 1..=2000) = read()
domain Player = 1..=n
let first: [Player]Player = read()

var queue: [Player]Deque<Player> = [Player] p => Deque.of([first[p]])
var last = first
for _ in 0..t:
    var hits: List<(to: Player, from: Player)> = []
    for p in Player:
        if let target = queue[p].pop_front():
            last[p] = target
            hits.push((to: target, from: p))
    for h in hits:
        queue[h.to].push(h.from)
print(last)
