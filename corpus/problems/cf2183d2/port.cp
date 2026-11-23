// Source: Codeforces 2183D2 (Tree Coloring, Hard Version),
//   https://codeforces.com/contest/2183/problem/D2
// Problem: a tree on n <= 2e5 vertices (sum), rooted at 1. An operation colours a set of
//   vertices with pairwise distinct depths and no edge between them. Print the fewest
//   operations that colour every vertex once, and the sets. Multitest.
// Solution: the answer is max over depths of (layer size + 1 if the whole layer, below the
//   root, hangs off one parent): that parent conflicts with all of it. Build it bottom-up,
//   slots = operations: in each layer, vertices with a child in a slot (all distinct slots)
//   take another slot - one not used one layer down if there is a single such vertex, else
//   a cyclic rotation of their forbidden slots - and the rest fill slots unused at this
//   depth. O(n * ans) worst case for the free-slot scans.

@!multitest
let n: 2..=200_000 = read()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](V, V)>())
var par: [V]?V = None
var by_depth: [0..n]List<V> = []
fn dfs(u: V, p: ?V, depth: Int):
    par[u] = p
    by_depth[depth].push(u)
    for nb in g[u] if Some(nb) != p: dfs(nb, u, depth + 1)
dfs(V.first!, None, 0)

let ans = [
    layer.len + Int(
        d > 0 and layer.all(u => par[u] == par[layer[0]])
    ) for (d, layer) in by_depth.items if not layer.is_empty
].max()!

domain Slot = 0..ans
var resp: [Slot]List<V> = []
var last_depth: [Slot]?Int = None
var forbidden_loc: [V]?Slot = None

for (d, layer) in by_depth.items.rev if not layer.is_empty:
    fn place(u: V, loc: Slot):
        resp[loc].push(u)
        last_depth[loc] = d
        if let p = par[u]: forbidden_loc[p] = loc

    let forbiddens = [(u:, loc: fl) for u in layer if let fl = forbidden_loc[u]]
    if forbiddens.len == 1:
        let s = Slot.find(x => last_depth[x] != Some(d + 1))!
        place(forbiddens[0].u, s)
    elif forbiddens.len >= 2:
        let rotated = forbiddens.map(f => f.loc).rotated_left(1)
        for k in 0..forbiddens.len: place(forbiddens[k].u, rotated[k])

    let unconstrained = [u for u in layer if forbidden_loc[u] == None]
    let free_slots = [s for s in Slot if last_depth[s] != Some(d)]
    for k in 0..min(unconstrained.len, free_slots.len): place(unconstrained[k], free_slots[k])

print(ans)
for group in resp:
    print(group.len, group)
