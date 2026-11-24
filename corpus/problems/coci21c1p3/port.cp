// Source: COCI 2021/2022 Contest 1 P3 (Logičari), https://dmoj.ca/problem/coci21c1p3
// limits: 1000 ms, 512 MB
// Problem: a connected graph with n <= 1e5 vertices and n edges. Colour the fewest vertices
//   blue so that every vertex has exactly one blue neighbour; print that count, or -1.
// Solution: the graph is a tree plus one edge (a, b), found with a DSU. Root the tree at a
//   and guess the colours of a and b; a tree DP over (vertex, its colour, parent's colour)
//   counts blues so each vertex gets exactly one blue among its parent, its children and,
//   for a or b, the other end of the extra edge. O(n) times 4 guesses.
// original: corpus/submissions/4080960.cpp
// Bounds: n: 1..=1e5; edges.u, edges.v: 1..=n

let n: 1..=1e5 = read()
domain V = 1..=n
domain E = 0..n
let edges: [E](u: V, v: V) = read()

// n edges on n connected vertices: a tree plus one extra edge (a, b)
var joined = Dsu<V>()
var extra: ?E = None
for e in E if not joined.unite(edges[e].u, edges[e].v): extra = e
let cut = extra!
let (a, b) = edges[cut]
let g = Graph.undirected(edges)
let tree = Rooted(v => [x.to for x in g.out(v) if x.e != cut], a)

fn plus(x: ?Int, y: ?Int) -> ?Int:
    if x == None or y == None: return None
    x! + y!

// With a's and b's colours guessed: the fewest blues in v's subtree, by v's colour and its
// parent's, so that every vertex below v and v itself end with one blue neighbour.
fn fewest(a_on: Bool, b_on: Bool) -> ?Int:
    var blues: [V, Bool, Bool]?Int = None
    for v in tree.order.rev:
        for on in Bool:
            if (v == a and on != a_on) or (v == b and on != b_on): continue
            let across = (v == a and b_on) or (v == b and a_on)  // a blue neighbour on the extra edge
            var no_kid: ?Int = Int(on)  // no blue child so far
            var one_kid: ?Int = None    // exactly one blue child
            for c in tree.children(v):
                one_kid = min(plus(one_kid, blues[c, false, on]), plus(no_kid, blues[c, true, on]))
                no_kid = plus(no_kid, blues[c, false, on])
            blues[v, on, false] = if across: no_kid else: one_kid
            if not across: blues[v, on, true] = no_kid
    blues[a, a_on, false]

var best: ?Int = None
for a_on in Bool:
    for b_on in Bool: best = min(best, fewest(a_on, b_on))
print(best ?? -1)
