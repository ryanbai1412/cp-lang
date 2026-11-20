// IOI 2012 Day 1 P2, Parachute Rings (buildable today-form)
// original: corpus/submissions/4689474.cpp (grader functions Init / Link / CountCritical)
// stdin: n and the number of calls, then one call per line: "u v" for
// Link(u, v), "-1 -1" for CountCritical. stdout: one count per CountCritical.
let n: 1..=1000000 = read()
let calls: 0..=1000000 = read()

// Layer 0 is the whole graph. Once some ring has three links, only it and
// its three neighbours can be critical; layer k = 1..4 is the graph without
// candidate k - 1, and the candidate survives while that graph is all chains.
var up: [0..5 * n]Int = -1                 // union-find: -size at a root
var deg: [0..5 * n]Int = 0
var adj: List<List<Int>> = []
for _ in 0..n: adj.push([])
var links: List<Int> = []
var cand: List<Int> = []
var chains: List<Bool> = [true, true, true, true]
var cycles = 0
var cycle_len = 0

fn find(x: Int) -> Int:
    var r = x
    while up[r] >= 0: r = up[r]
    var y = x
    while up[y] >= 0:
        let next = up[y]
        up[y] = r
        y = next
    r

// joins u and v in one layer; false when they were already connected
fn unite(layer: Int, u: Int, v: Int) -> Bool:
    var a = find(layer * n + u)
    var b = find(layer * n + v)
    if a == b: return false
    if up[a] > up[b]:
        swap(mut a, mut b)
    up[a] += up[b]
    up[b] = a
    true

fn link_without(k: Int, u: Int, v: Int):
    if u == cand[k] or v == cand[k]: return
    let layer = k + 1
    deg[layer * n + u] += 1
    deg[layer * n + v] += 1
    if deg[layer * n + u] > 2 or deg[layer * n + v] > 2: chains[k] = false
    if not unite(layer, u, v): chains[k] = false

fn link(u: Int, v: Int):
    links.push(u)
    links.push(v)
    if cand.len > 0:
        for k in 0..4: link_without(k, u, v)
        return
    adj[u].push(v)
    adj[v].push(u)
    deg[u] += 1
    deg[v] += 1
    if deg[u] == 3 or deg[v] == 3:
        let hub = if deg[u] == 3: u else: v
        cand.push(hub)
        for x in adj[hub]: cand.push(x)
        for k in 0..4:
            for e in 0..links.len / 2: link_without(k, links[2 * e], links[2 * e + 1])
    elif not unite(0, u, v):
        // all degrees are at most 2, so this component just became a cycle
        cycles += 1
        cycle_len = -up[find(u)]

fn count_critical() -> Int:
    if cand.len > 0: return count(chains[k] for k in 0..4)
    if cycles == 0: return n
    if cycles == 1: return cycle_len
    0

for _ in 0..calls:
    let u: Int = read()
    let v: Int = read()
    if u == -1: print(count_critical())
    else: link(u, v)
