// original: corpus/submissions/5445683.cpp
// NOIP 2022 Barracks: a connected country of cities and roads. Build
// barracks in a non-empty set of cities and guard a set of roads, so that
// whichever single unguarded road the enemy cuts, all the barracks can
// still reach each other. Count the choices.
use std.graph.{two_edge_components, condense}
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

// Cutting every bridge leaves blocks that no single cut can split; the
// blocks form a tree joined by the bridges.
let (domain Block, block) = two_edge_components(land)
var cities: [Block]Int = 0
for v in City: cities[block[v]] += 1
let blocks = Rooted(condense(land, block), block[City.first!])

// Count as if all m roads were free to guard or not, and halve once for
// every bridge that has to be guarded.
// held: non-empty sets of barracks in the blocks below b's bridge and on,
// with every bridge between them and b guarded.
var some: [Block]Z = 1  // over b's child blocks: no barracks there, or a held set and its bridge
var one: [Block]Z = 0   // sets that lie inside a single child block and its bridge
var total: Z = 0
let two = Z(2)
for b in blocks.at.rev:
    let held = two ** cities[b] * some[b] - 1
    // the sets whose highest block is b: all but those held through one child alone
    total += held - one[b]
    if let p = blocks.parent(b):
        some[p] *= 1 + held / 2
        one[p] += held / 2
print(total * two ** m)
