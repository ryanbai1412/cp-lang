// Mock CCC '22 Contest 1 J4 — Simultaneous Target-Queue Simulation (CP Lang v2 spec)
// original: corpus/submissions/4223440.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Each of n players starts with one target in a FIFO queue. For t simultaneous ticks,
// every player with a non-empty queue throws to its front target and records it as
// `last`; all throws in tick c arrive at the backs of their targets' queues after
// everyone has thrown (in ascending thrower order).

let (n, t): (1..=2000, 1..=2000) = read()
domain Player = 1..=n
let init: [Player]Player = read()

var q: [Player]Deque<Player> = [Player] p => [init[p]]
var last: [Player]?Player = none

for _ in 0..t:
    var arrivals: List<(to: Player, from: Player)> = []
    for p in Player:
        if let target = q[p].pop_front():
            last[p] = target
            arrivals.push((to: target, from: p))
    for (to, from) in arrivals:
        q[to].push(from)

print([last[p]! for p in Player], sep: " ")
