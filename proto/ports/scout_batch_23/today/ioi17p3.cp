// IOI '17 P3 — Toy Train (CP Lang 0.2.2): `Marks<V>` (O(1) clear) + `Deque<V>`
// replace 4x per-iteration `List.filled(n, false)` resets and 2x manual queues.
// original: corpus/submissions/5656304.cpp
use std.deque.{Deque}
use std.marks.{Marks}

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

print([if alive[v]: 1 else: 0 for v in V])
