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

@!multitest
let n: 2..=200_000 = read()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<[n - 1](V, V)>()), V.first)
let by_depth = V.group(by: t.depth)

let ans = [
    layer.len + Int(d > 0 and layer.all(u => t.parent(u) == t.parent(layer[0])))
    for (d, layer) in by_depth.items
].max()!

domain Slot = 0..ans
var resp: [Slot]List<V> = []
var last_depth: [Slot]?Int = None
var forbidden_loc: [V]?Slot = None

for (d, layer) in by_depth.items.rev:
    fn place(u: V, loc: Slot):
        resp[loc].push(u)
        last_depth[loc] = d
        if let par = t.parent(u): forbidden_loc[par] = loc

    let forbiddens = [(u:, loc: fl) for u in layer if let fl = forbidden_loc[u]]
    if forbiddens.len == 1:
        let s = Slot.first(x => last_depth[x] != Some(d + 1))!
        place(forbiddens[0].u, s)
    elif forbiddens.len >= 2:
        let rotated = forbiddens.map(f => f.loc).rotated_left(1)
        for (f, loc) in zip(forbiddens, rotated):
            place(f.u, loc)

    let unconstrained = [u for u in layer if forbidden_loc[u] == None]
    let free_slots = [s for s in Slot if last_depth[s] != Some(d)]
    for (u, s) in zip(unconstrained, free_slots):
        place(u, s)

print(ans)
for group in resp:
    print(group.len, group)
