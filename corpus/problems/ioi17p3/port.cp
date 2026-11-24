// Source: IOI 2017 P3 Toy Train, https://dmoj.ca/problem/ioi17p3
// Problem: a directed graph (n <= 5000 stations, m <= 20000 tracks, self-loops allowed, every
//   station has an out-track); each station is owned by Arezou or Borzou and may be a charging
//   station. The owner of the current station picks the next track; Arezou wins iff charging
//   stations are visited infinitely often. For each start station print whether Arezou wins.
// Solution: repeat on the stations still alive: good = Arezou's attractor of "reach a charging
//   station in >= 1 step" (her stations need one edge into it, Borzou's all live out-edges),
//   by BFS over reverse edges. If every live station is good, stop. Otherwise Borzou's
//   attractor of the non-good stations loses for Arezou: remove it, update live out-degrees
//   and repeat. Marks give O(1) clears; O(n (n + m)) in all.
// original: corpus/submissions/5656304.cpp
// Bounds: n: 1..=5000; m: n..=20_000; owner, charge: 0..=1; tracks.from, tracks.to: 0..=n - 1
let n: 1..=5000 = read()
let m: n..=20_000 = read()
domain V = 0..n
let owner: [V]0..=1 = read()  // 1: Arezou's station, 0: Borzou's
let charge: [V]0..=1 = read()
let tracks: [m](from: V, to: V) = read()
var into: [V]List<V> = []
for t in tracks: into[t.to].push(t.from)

// the live stations whose owner-side `player` forces the train, in one step or more, into a live
// station of `target`: the player's stations need one track there, the other's all live tracks
fn attract(target: [V]Bool, player: Int, alive: [V]Bool) -> [V]Bool:
    var exits: [V]Int = 0
    for t in tracks:
        if alive[t.from] and alive[t.to]: exits[t.from] += 1
    var won: [V]Bool = false
    var queued: [V]Bool = [V] v => alive[v] and target[v]
    var queue = Deque<V>.new()
    for v in V:
        if queued[v]: queue.push(v)
    while let w = queue.pop_front():
        for u in into[w]:
            if not alive[u] or won[u]: continue
            exits[u] -= 1
            won[u] = owner[u] == player or exits[u] == 0
            if won[u] and not queued[u]:
                queued[u] = true
                queue.push(u)
    won

// stations that can't reach a charge again infinitely often lose, and so does all Borzou can
// force into them; drop them and repeat on what is left
var alive: [V]Bool = true
while true:
    let good = attract([V] v => charge[v] == 1, 1, alive)
    let bad: [V]Bool = [V] v => alive[v] and not good[v]
    if not V.any(v => bad[v]): break
    let lost = attract(bad, 0, alive)
    for v in V:
        if bad[v] or lost[v]: alive[v] = false

print([Int(alive[v]) for v in V])
