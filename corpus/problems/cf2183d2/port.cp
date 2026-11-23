// Codeforces 2183 D2: Tree Coloring.
// Partition vertices of a rooted tree into `ans` slots so no two vertices at
// the same depth share a slot and no parent shares a slot with its child.
// Lower bound: a depth `d` with `sz` vertices needs `>= sz` slots, and if
// `d > 0` and all `sz` vertices share one parent `p0`, `p0` cannot use any of
// those slots, forcing `>= sz + 1`. Thus `ans = max_d (sz + Int(same_parent))`
// is necessary and sufficient.
// Bottom-up by depth `d`, vertices whose children occupy a slot at depth `d + 1`
// form `forbiddens` with pairwise distinct forbidden slots:
//   - If `forbiddens.len == 1`, at least one slot was unused at depth `d + 1`.
//   - If `forbiddens.len >= 2`, cyclically rotating their forbidden slots via
//     `rotated_left(1)` gives every vertex a slot distinct from its own.
// Remaining vertices at depth `d` take the remaining slots unused at depth `d`.

use std.seq.rotated_left

@!multitest
let n: 2..=200_000 = read()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](V, V)>())
var par: [V]?V = none
var by_depth: [0..n]List<V> = []
fn dfs(u: V, p: ?V, depth: Int):
    par[u] = p
    by_depth[depth].push(u)
    for nb in g[u] if some(nb) != p: dfs(nb, u, depth + 1)
dfs(V.first!, none, 0)

let ans = [
    layer.len + Int(
        d > 0 and layer.all(u => par[u] == par[layer[0]])
    ) for (d, layer) in by_depth.items if not layer.is_empty
].max()!

domain Slot = 0..ans
var resp: [Slot]List<V> = []
var last_depth: [Slot]?Int = none
var forbidden_loc: [V]?Slot = none

for (d, layer) in by_depth.items.rev if not layer.is_empty:
    fn place(u: V, loc: Slot):
        resp[loc].push(u)
        last_depth[loc] = d
        if let p = par[u]: forbidden_loc[p] = loc

    let forbiddens = [(u:, loc: fl) for u in layer if let fl = forbidden_loc[u]]
    if forbiddens.len == 1:
        let s = Slot.first(x => last_depth[x] != some(d + 1))!
        place(forbiddens[0].u, s)
    elif forbiddens.len >= 2:
        let rotated = forbiddens.map(f => f.loc).rotated_left(1)
        for k in 0..forbiddens.len: place(forbiddens[k].u, rotated[k])

    let unconstrained = [u for u in layer if forbidden_loc[u] == none]
    let free_slots = [s for s in Slot if last_depth[s] != some(d)]
    for k in 0..min(unconstrained.len, free_slots.len): place(unconstrained[k], free_slots[k])

print(ans)
for group in resp:
    print(group.len, group)
