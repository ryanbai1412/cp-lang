// IOI '17 P3 - Toy Train (DMOJ: ioi17p3, 30pt, TL 1.0s, ML 256MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Buchi / retrograde game on a directed graph (N stations 0..N-1, M tracks).
// Each station i has owner a[i] (1 = Arezou, 0 = Bita) and charging flag r[i] (1 = charging).
// Arezou wins iff she can force visiting charging stations infinitely often.
//
// Nested retrograde attractor fixed point:
//   Maintain Bita-closed set `alive` (initially all stations).
//   1) Compute `good` = alive stations from which Arezou can force reaching an alive
//      charging station in >= 1 step (reverse BFS from alive charging stations).
//   2) Let `bad = alive \ good`. If empty, Arezou wins from every station in `alive`.
//   3) Otherwise Bita wins from `bad`; compute `lose` = Bita-attractor of `bad` in `alive`,
//      remove `lose` from `alive`, and repeat.
// WISH MG14: IOI grader-style function interface `pub fn who_wins(...)` vs stdin/stdout
let (n, m): (1..=5000, 1..=20000) = read()
domain V = 0..n
domain E = 0..m
// WISH MG11: graph constraint — every station must have out-degree >= 1
let owner: [V](0..=1) = read()
let charge: [V](0..=1) = read()
let tracks: [E](u: V, v: V) = read()

var rev: [V]List<V> = []
var init_out: [V]Int = 0
for e in E:
    let (u, v) = (tracks[e].u, tracks[e].v)
    rev[v].push(u)
    init_out[u] += 1

var alive: [V]Bool = true
var out_alive: [V]Int = [V] v => init_out[v]

while true:
    // Step 1: Arezou's >= 1-step attractor of alive charging stations inside `alive`
    var good: [V]Bool = false
    var pushed: [V]Bool = false
    var rem: [V]Int = [V] v => out_alive[v]
    var q: List<V> = []
    for v in V:
        if alive[v] and charge[v] == 1:
            pushed[v] = true
            q.push(v)
    var head = 0
    while head < q.len:
        let w = q[head]
        head += 1
        for u in rev[w]:
            if not alive[u] or good[u]: continue
            if owner[u] == 1:
                good[u] = true
            else:
                rem[u] -= 1
                if rem[u] == 0: good[u] = true
            if good[u] and not pushed[u]:
                pushed[u] = true
                q.push(u)

    // Step 2: collect `bad = alive \ good`; if empty, fixed point reached
    var lose: [V]Bool = false
    var lq: List<V> = []
    for v in V:
        if alive[v] and not good[v]:
            lose[v] = true
            lq.push(v)
    if lq.is_empty: break

    // Step 3: Bita's attractor of `bad` inside `alive`
    var lhead = 0
    while lhead < lq.len:
        let w = lq[lhead]
        lhead += 1
        for u in rev[w]:
            if not alive[u] or lose[u]: continue
            if owner[u] == 0:
                lose[u] = true
                lq.push(u)
            else:
                out_alive[u] -= 1
                if out_alive[u] == 0:
                    lose[u] = true
                    lq.push(u)

    for v in lq:
        alive[v] = false
        for u in rev[v]:
            if alive[u] and owner[u] == 0:
                out_alive[u] -= 1

print([if alive[v]: 1 else: 0 for v in V])
