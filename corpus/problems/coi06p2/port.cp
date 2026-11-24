// Source: COI 2006 (Policija), https://dmoj.ca/problem/coi06p2
// Problem: a connected graph of N <= 1e5 towns and E <= 5e5 roads; Q <= 3e5 queries: can one get
//   from A to B with (1) the road G1-G2 closed, or (2) town C closed? Print yes or no.
// Solution: in a DFS tree every other road joins a town to an ancestor; low[v] is the highest
//   ancestor (smallest preorder position) such a road reaches from v's subtree. Closing a road
//   cuts only a bridge (a tree edge whose bottom has low not above it), separating A from B iff
//   exactly one is under the bottom. Closing C strands the child subtree of C holding x iff
//   low[child] is not above C; A and B stay joined iff they are stranded in the same piece or
//   both not at all. O(N + E + Q log N).
// Bounds: n: 2..=1e5; m: 1..=5e5; roads.u, roads.v: 1..=n; q: 1..=3e5
with io(answers: ("yes", "no"))
let (n, m): (2..=1e5, 1..=5e5) = read()
domain Town = 1..=n
domain Road = 0..m
let roads: [Road](u: Town, v: Town) = read()
let map = Graph.undirected(roads)
let (tree, tree_road) = map.dfs_tree(Town.first!)
var low = [Town] v => tree.pos(v)
for v in tree.at.rev:
    for a in map.out(v):
        if not tree_road[a.e]: low[v] min= tree.pos(a.to)
    if let p = tree.parent(v): low[p] min= low[v]

// With town c closed, the child subtree of c that x is cut off in, if any.
fn stranded(c: Town, x: Town) -> ?Town:
    if not tree.is_anc(c, x) or c == x: return None
    let child = tree.jump(c, x, 1)!
    if low[child] < tree.pos(c): None else: Some(child)

// With road u-v closed: is it a bridge with exactly one of a and b under its bottom end?
fn separates(u: Town, v: Town, a: Town, b: Town) -> Bool:
    let bottom = if tree.parent(v) == Some(u): v elif tree.parent(u) == Some(v): u else: return false
    low[bottom] >= tree.pos(bottom) and tree.is_anc(bottom, a) != tree.is_anc(bottom, b)

let q: 1..=3e5 = read()
repeat q:
    let (kind, a, b): (1..=2, Town, Town) = read()
    if kind == 1:
        let (u, v): (Town, Town) = read()
        print(yes(not separates(u, v, a, b)))
    else:
        let c: Town = read()
        print(yes(stranded(c, a) == stranded(c, b)))
