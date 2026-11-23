// DMOPC '21 Contest 6 P4: colour each road 1 or 2. Roads at c get colour 1,
// roads at d colour 2 (a road joining c and d is impossible). A breadth-first
// search over (city, which of c and d were met) finds the nearest way to
// reach b without meeting both; the other roads take the colour it needs.
// Runnable on proto/cpc today.
// original: corpus/submissions/4342347.cpp
let n: 1..=2e5 = read()
let m: 1..=2e5 = read()
domain V = 1..=n
let edges: [0..m](V, V) = read()
let a: V = read()
let b: V = read()
let c: V = read()
let d: V = read()
domain Seen = 0..4  // bit 0: met c, bit 1: met d

fn mark(u: V, seen: Seen) -> Seen:
    var s = seen.idx
    if u == c and s % 2 == 0: s += 1
    if u == d and s < 2: s += 2
    Seen(s)

// No top-level early exit on the prototype, so the program lives in a function.
fn solve():
    var colour: [0..m]Int = 0
    var adj: [V]List<V> = []
    for i in 0..m:
        let (u, v) = edges[i]
        adj[u].push(v)
        adj[v].push(u)
        let at_c = u == c or v == c
        let at_d = u == d or v == d
        if at_c and at_d:
            print(-1)
            return
        if at_c: colour[i] = 1
        elif at_d: colour[i] = 2

    var dist: [V, Seen]?Int = None
    var queue: List<(V, Seen)> = [(a, mark(a, Seen(0)))]
    dist[a, mark(a, Seen(0))] = 0
    var head = 0
    while head < queue.len:
        let (u, s) = queue[head]
        head += 1
        for v in adj[u]:
            let s2 = mark(v, s)
            if dist[v, s2] == None:
                dist[v, s2] = dist[u, s]! + 1
                queue.push((v, s2))

    var best: ?(Int, Int) = None
    for s in 0..3:
        if let x = dist[b, Seen(s)]:
            if best == None or x < best!.0: best = (x, s)
    if best == None:
        print(-1)
        return
    let rest = if best!.1 >= 2: 2 else: 1
    print(2)
    for i in 0..m: print(if colour[i] == 0: rest else: colour[i])

solve()
