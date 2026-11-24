// Source: COI 2006 (Policija), https://dmoj.ca/problem/coi06p2
// Problem: a connected graph, N <= 1e5 towns and E <= 5e5 roads. Q <= 3e5 queries: can one get
//   from A to B with (1) the road G1-G2 closed, or (2) town C closed? Print yes or no.
// Solution: a DFS tree (built by hand: Rooted is breadth-first) with low[v], the earliest
//   preorder position a non-tree road reaches from v's subtree. A road matters only if it is a
//   tree edge with low[bottom] > pos[top] and A, B on opposite sides of bottom's subtree.
//   Closing C cuts off the child subtree of C containing x iff low[child] >= pos[C]; A and B
//   stay connected iff they land in the same cut-off piece or both outside. O(N + E + Q log N).
let (n, m): (2..=1e5, 1..=5e5) = read()
domain Town = 1..=n
domain Road = 0..m
let roads: [Road](u: Town, v: Town) = read()
let map = Graph.undirected(roads)

// In a depth-first tree every other road joins a town to one of its
// ancestors. low[v]: the earliest town such a road reaches from v's subtree.
// `Rooted(map, ..)` is breadth-first, so search depth-first by hand and root
// the tree it leaves (`map.dfs_tree` once std has it).
let adj = [Town] v => [w for (w, _) in map.out(v)]
var kids = [Town] _ => List<Town>()
var seen = [Town] v => v == Town.first!
var tried = [Town] _ => 0
var path = [Town.first!]
while let v = path.last:
    if tried[v] == adj[v].len:
        path.pop()
        continue
    let w = adj[v][tried[v]]
    tried[v] += 1
    if not seen[w]:
        seen[w] = true
        kids[v].push(w)
        path.push(w)
let tree = Rooted(kids, Town.first!)
var low = [Town] v => tree.pos(v)
for v in tree.at.rev:
    for (w, _) in map.out(v):
        if tree.parent(v) != Some(w): low[v] min= tree.pos(w)
    if let p = tree.parent(v): low[p] min= low[v]

// With town c closed, the part of c's subtree that x is cut off in, if any.
fn stranded(c: Town, x: Town) -> ?Town:
    if not tree.is_anc(c, x): return None
    let child = tree.jump(c, x, 1)!
    if low[child] < tree.pos(c): None else: child

fn answer(ok: Bool): print(if ok: "yes" else: "no")

let q: 1..=3e5 = read()
for _ in 0..q:
    let (kind, a, b): (1..=2, Town, Town) = read()
    if kind == 1:
        let (u, v): (Town, Town) = read()
        let (top, bottom) = if tree.is_anc(u, v): (u, v) else: (v, u)
        // closing a road matters only if it is a bridge with a and b on opposite sides
        answer(low[bottom] <= tree.pos(top) or tree.is_anc(bottom, a) == tree.is_anc(bottom, b))
    else:
        let c: Town = read()
        answer(stranded(c, a) == stranded(c, b))
