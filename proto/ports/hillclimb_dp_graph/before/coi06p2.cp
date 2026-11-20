// cp: 0.1 0f8d77b+dirty
// COI 2006 Policija: a connected road map. Each query closes one road or
// one town and asks whether the police can still drive from town a to b.
let n: 2..=1e5 = read()
let m: n - 1..=5e5 = read()
domain Town = 1..=n
domain Road = 0..m

fn connected() -> [Road](u: Town, v: Town):  // generator: a random tree, then extra roads
    [Road] r => (
        if r.idx < n - 1: (
            Town(rand(1, r.idx + 1)),
            Town(r.idx + 2),
        ) else: (Town(rand(1, n)), Town(rand(1, n)))
    )

let roads: [Road](u: Town, v: Town) = read(gen: connected)
var map: [Town]List<Town> = []
for (u, v) in roads:
    map[u].push(v)
    map[v].push(u)

// Depth-first tree: every other road joins a town to one of its ancestors.
// low[v]: the earliest town such a road reaches from v's subtree.
var enter: [Town]Int = -1
var leave: [Town]Int = -1
var low: [Town]Int = 0
var kids: [Town]List<Town> = []
var clock = 0
fn explore(u: Town, from: ?Town):
    enter[u] = clock
    low[u] = clock
    clock += 1
    for w in map[u]:
        if enter[w] < 0:
            kids[u].push(w)
            explore(w, u)
            low[u] min= low[w]
        elif from != w: low[u] min= enter[w]
    leave[u] = clock
explore(Town.first, none)

fn is_anc(a: Town, b: Town) -> Bool: enter[a] <= enter[b] and leave[b] <= leave[a]

// With town c closed, the part of c's subtree that x is cut off in, if any.
fn stranded(c: Town, x: Town) -> ?Town:
    if x == c or not is_anc(c, x): return none
    var lo = 0  // the last child entered no later than x
    var hi = kids[c].len - 1
    while lo < hi:
        let mid = (lo + hi + 1) / 2
        if enter[kids[c][mid]] <= enter[x]: lo = mid else: hi = mid - 1
    let child = kids[c][lo]
    if low[child] < enter[c]: none else: child

let q: 1..=3e5 = read()
for _ in 0..q:
    let kind: 1..=2 = read()
    let a: Town = read()
    let b: Town = read()
    if kind == 1:
        let u: Town = read()
        let v: Town = read()
        let (top, bottom) = if is_anc(u, v): (u, v) else: (v, u)
        // closing a road matters only if it is a bridge with a and b on opposite sides
        let ok = low[bottom] <= enter[top] or is_anc(bottom, a) == is_anc(bottom, b)
        print(if ok: "yes" else: "no")
    else:
        let c: Town = read()
        assert c != a and c != b
        print(if stranded(c, a) == stranded(c, b): "yes" else: "no")
