// Source: IOI 2012 Day 1 Parachute Rings, https://dmoj.ca/problem/ioi12p2
// Problem: n <= 1e6 rings, then up to 1e6 calls in total: Link(a, b) joins two distinct rings not
//   yet linked; CountCritical() asks how many rings leave nothing but chains (paths, single rings
//   included) when removed.
// Solution: while all degrees are <= 2, a DSU tracks cycles: none means all n are critical, one
//   means its rings, two or more means none. The first ring to reach degree 3 and its three
//   neighbours are the only candidates; for each keep a DSU and degrees of the graph without it,
//   replaying all links so far and then each new one, and count the candidates whose graph has
//   no degree > 2 and no cycle. O((n + calls) alpha(n)): std Dsu for the whole graph and one per candidate.
// limits: 2500 ms, 256 MB
// original: corpus/submissions/4689474.cpp (grader functions Init / Link / CountCritical)
// stdin: n and the number of calls, then one call per line: "u v" for
// Link(u, v), "-1 -1" for CountCritical. stdout: one count per CountCritical.
// Bounds: n: 1..=1000000; calls: 0..=1000000
let n: 1..=1000000 = read()
let calls: 0..=1000000 = read()
domain Ring = 0..n
domain Cand = 0..4  // the hub, then its three neighbours

// While every ring has at most two links, `whole` tracks the cycles.
var whole = Dsu<Ring>()
var deg: [Ring]Int = 0
var adj: [Ring]List<Ring> = []
var links: List<(Ring, Ring)> = []
var cycles = 0
var cycle_len = 0

// Once a ring (the hub) has three links, only it and its neighbours can be
// critical: candidate k survives while the graph without it is all chains.
var hub: ?Ring = None
var without: [Cand]Dsu<Ring> = [Cand] _ => Dsu()
var deg_without: [Cand, Ring]Int = 0
var chains: [Cand]Bool = true

fn cand(h: Ring, k: Cand) -> Ring = if let i = k.prev: adj[h][i.ord] else: h

fn link_without(h: Ring, k: Cand, u: Ring, v: Ring):
    let c = cand(h, k)
    if u == c or v == c: return
    for x in [u, v]: deg_without[k, x] += 1
    if deg_without[k, u] > 2 or deg_without[k, v] > 2 or not without[k].unite(u, v):
        chains[k] = false

fn link(u: Ring, v: Ring):
    links.push((u, v))
    if let h = hub:
        for k in Cand: link_without(h, k, u, v)
        return
    for (a, b) in [(u, v), (v, u)]:
        adj[a].push(b)
        deg[a] += 1
    if deg[u] == 3 or deg[v] == 3:
        let h = if deg[u] == 3: u else: v
        hub = Some(h)
        for k in Cand:
            for (a, b) in links: link_without(h, k, a, b)
    elif not whole.unite(u, v):
        cycles += 1
        cycle_len = whole.size(u)

fn count_critical() -> Int:
    if hub != None: return count(chains[k] for k in Cand)
    if cycles == 0: return n
    if cycles == 1: return cycle_len
    0

for _ in 0..calls:
    let (u, v): (-1..=n - 1, -1..=n - 1) = read()
    if u == -1: print(count_critical())
    else: link(Ring(u), Ring(v))
