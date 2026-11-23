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

let (n, m): (Int, Int) = read()
domain V = 0..n
let owner: [V]Int = read()
let charge: [V]Int = read()

var rev: [V]List<V> = []
var out_alive: [V]Int = 0
for _ in 0..m:
    let (u, v): (V, V) = read()
    rev[v].push(u)
    out_alive[u] += 1

var alive: [V]Bool = true
var good = Marks<V>()
var pushed = Marks<V>()
var lose = Marks<V>()
var q = Deque<V>.new()
var lq = Deque<V>.new()

while true:
    good.clear()
    pushed.clear()
    var rem: [V]Int = [V] v => out_alive[v]
    for v in V:
        if alive[v] and charge[v] == 1:
            pushed.add(v)
            q.push(v)
    while let w = q.pop_front():
        for u in rev[w]:
            if not alive[u] or u in good: continue
            if owner[u] == 1:
                good.add(u)
            else:
                rem[u] -= 1
                if rem[u] == 0: good.add(u)
            if u in good and u not in pushed:
                pushed.add(u)
                q.push(u)

    lose.clear()
    var lost: List<V> = []
    for v in V:
        if alive[v] and v not in good:
            lose.add(v)
            lq.push(v)
            lost.push(v)
    if lost.is_empty: break

    while let w = lq.pop_front():
        for u in rev[w]:
            if not alive[u] or u in lose: continue
            if owner[u] == 0:
                lose.add(u)
                lq.push(u)
                lost.push(u)
            else:
                out_alive[u] -= 1
                if out_alive[u] == 0:
                    lose.add(u)
                    lq.push(u)
                    lost.push(u)

    for v in lost:
        alive[v] = false
        for u in rev[v]:
            if alive[u] and owner[u] == 0:
                out_alive[u] -= 1

print([Int(alive[v]) for v in V])
