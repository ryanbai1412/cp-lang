// original: corpus/submissions/5445683.cpp
// NOIP 2022 Barracks: a connected country of cities and roads. Build
// barracks in a non-empty set of cities and guard a set of roads, so that
// whichever single unguarded road the enemy cuts, all the barracks can
// still reach each other. Count the choices.
const P = 1000000007
type Z = Mod<P>
let n: 1..=5e5 = read()
let m: n - 1..=1e6 = read()
domain City = 1..=n
domain Road = 0..m

fn connected() -> [Road](u: City, v: City):  // generator: a random tree, then extra roads
    [Road] r => (
        if r.idx < n - 1: (
            City(rand(1, r.idx + 1)),
            City(r.idx + 2),
        ) else: (City(rand(1, n)), City(rand(1, n)))
    )

let roads: [Road](u: City, v: City) = read(gen: connected)
let land = Graph.undirected(roads)

// In a depth-first tree every other road joins a city to one of its
// ancestors. low[v]: the earliest city such a road reaches from v's subtree.
// The road above v is a bridge when nothing reaches past v.
let tree = Rooted(land, City.first)
var low: [City]Int = [City] v => tree.pos(v)
for v in tree.at.rev:
    for w in land[v]:
        if tree.parent(v) != w: low[v] min= tree.pos(w)
    if let p = tree.parent(v): low[p] min= low[v]

// Cutting every bridge leaves blocks that no single cut can split. Name a
// block by its topmost city; the blocks form a tree joined by the bridges.
var top: [City]City = [City] v => v
var cities: [City]Int = 0
for v in tree.at:
    if let p = tree.parent(v) and low[v] < tree.pos(v): top[v] = top[p]
    cities[top[v]] += 1

// Count as if all m roads were free to guard or not, and halve once for
// every bridge that has to be guarded.
// held: non-empty sets of barracks in the blocks below b's bridge and on,
// with every bridge between them and b guarded.
var some: [City]Z = 1  // over b's child blocks: no barracks there, or a held set and its bridge
var one: [City]Z = 0   // sets that lie inside a single child block and its bridge
var total: Z = 0
let two = Z(2)
for b in tree.at.rev:
    if top[b] != b: continue
    let held = two ** cities[b] * some[b] - 1
    // the sets whose highest block is b: all but those held through one child alone
    total += held - one[b]
    if let p = tree.parent(b):
        some[top[p]] *= 1 + held / 2
        one[top[p]] += held / 2
print(total * two ** m)
