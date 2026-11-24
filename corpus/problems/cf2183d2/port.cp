// Source: Codeforces 2183D2 (Tree Coloring, Hard Version),
//   https://codeforces.com/contest/2183/problem/D2
// limits: 2000 ms, 512 MB
// Problem: a tree on n <= 2e5 vertices (sum), rooted at 1. An operation colours a set of
//   vertices with pairwise distinct depths and no edge between them. Print the fewest
//   operations that colour every vertex once, and the sets. Multitest.
// Solution: slots = operations. A layer of sz vertices needs sz slots, and one more if it
//   lies below the root and hangs off a single parent, which conflicts with all of it; the
//   max over layers suffices. Fill layers bottom-up. A vertex whose child took slot s must
//   avoid s: one such vertex takes a slot unused one layer down; several swap their
//   forbidden slots by a cyclic rotation. The rest take free slots among the first sz,
//   where at most sz - (rest) are taken. O(n).

@!multitest
let n: 2..=200_000 = read()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<[n - 1](V, V)>()), V.first!)
let layers = t.bfs.runs(by: t.depth).collect()

let ans = [
    layer.len + Int(d > 0 and layer.all(u => t.parent(u) == t.parent(layer.first!)))
    for (d, layer) in layers.items
].max()!

domain Slot = 0..ans
var resp: [Slot]List<V> = []
var last_depth: [Slot]?Int = None
var forbidden_loc: [V]?Slot = None

fn place(u: V, loc: Slot, d: Int):
    resp[loc].push(u)
    last_depth[loc] = d
    if let p = t.parent(u): forbidden_loc[p] = loc

for (d, layer) in layers.items.rev:
    let forbiddens = [(u:, loc: fl) for u in layer if let fl = forbidden_loc[u]]
    if forbiddens.len == 1:
        place(forbiddens[0].u, Slot.find(s => last_depth[s] != Some(d + 1))!, d)
    elif forbiddens.len >= 2:
        for (f, loc) in zip(forbiddens, forbiddens.map(f => f.loc).rotated_left(1)):
            place(f.u, loc, d)

    let unconstrained = [u for u in layer if forbidden_loc[u] == None]
    let free_slots = [s for s in Slot.take(layer.len) if last_depth[s] != Some(d)]
    for (u, s) in zip(unconstrained, free_slots):
        place(u, s, d)

print(ans)
for group in resp:
    print(group.len, group)
