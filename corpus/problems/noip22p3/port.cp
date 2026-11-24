// Source: NOIP 2022 P3, Barracks, https://dmoj.ca/problem/noip22p3
// Problem: a connected simple graph, n <= 5e5 cities and m <= 1e6 roads. Choose a non-empty set
//   of barracks cities and a set of guarded roads so that cutting any one unguarded road keeps
//   all barracks connected. Count the choices mod 1e9+7.
// Solution: no single cut splits a 2-edge-connected block, so condense the blocks into the
//   bridge tree. Only bridges between barracks must be guarded: count as if all m roads were
//   free (2^m) and halve once per bridge on the subtree spanned by the barracks. A DP over the
//   rooted block tree counts held sets below each bridge and adds the sets whose highest
//   block is b, minus those held through one child alone. O(n + m).
// original: corpus/submissions/5445683.cpp
// Bounds: n: 1..=5e5; m: n - 1..=1e6
use std.graph.{two_edge_components}
const P = 1000000007
type Z = Mod<P>
let n: 1..=5e5 = read()
let m: n - 1..=1e6 = read()
domain City = 1..=n
domain Road = 0..m
let land = Graph.undirected(read<[Road](u: City, v: City)>())

// Cutting every bridge leaves blocks that no single cut can split; the
// blocks form a tree joined by the bridges.
let (domain Block, block) = two_edge_components(land)
var cities: [Block]Int = 0
for v in City: cities[block[v]] += 1
let blocks = Rooted(condense(land, block), root: block[City.first!])

// Count as if all m roads were free to guard or not, and halve once for
// every bridge that has to be guarded.
// held: non-empty sets of barracks in the blocks below b's bridge and on,
// with every bridge between them and b guarded.
var some: [Block]Z = 1  // over b's child blocks: no barracks there, or a held set and its bridge
var one: [Block]Z = 0   // sets that lie inside a single child block and its bridge
var total: Z = 0
let two = Z(2)
let half = two.inv()
for b in blocks.at.rev:
    let held = two ** cities[b] * some[b] - 1
    // the sets whose highest block is b: all but those held through one child alone
    total += held - one[b]
    if let p = blocks.parent(b):
        some[p] *= 1 + held * half
        one[p] += held * half
print(total * two ** m)
