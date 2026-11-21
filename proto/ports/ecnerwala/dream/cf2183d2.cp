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
//   - If `forbiddens.len >= 2`, cyclically rotating their forbidden slots gives
//     every vertex a slot distinct from its own forbidden slot.
// Remaining vertices at depth `d` take the remaining slots unused at depth `d`.

@!multitest
let n: 2..=200_000 = read()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<[n - 1](V, V)>()), V.first)

var by_depth: [0..n]List<V> = []
for p in t.Pre:
    let u = t.at[p]
    by_depth[t.depth(u)].push(u)

var ans = 0
for d in 0..n:
    if by_depth[d].is_empty: break
    let sz = by_depth[d].len
    let p0 = t.parent(by_depth[d][0])
    let same_parent = d > 0 and by_depth[d].all(u => t.parent(u) == p0)
    ans max= sz + Int(same_parent)

domain Slot = 0..ans
var resp: [Slot]List<V> = []
var last_depth: [Slot]?Int = none
var forbidden_loc: [V]?Slot = none

for d in (0..n).rev:
    if by_depth[d].is_empty: continue

    fn place(u: V, loc: Slot):
        resp[loc].push(u)
        last_depth[loc] = d
        if let par = t.parent(u): forbidden_loc[par] = loc

    var forbiddens: List<(u: V, loc: Slot)> = []
    for u in by_depth[d]:
        if let fl = forbidden_loc[u]: forbiddens.push((u, fl))

    if forbiddens.len == 1:
        let s = [x for x in Slot].first(x => last_depth[x] != some(d + 1))!
        place(forbiddens[0].u, s)
    elif forbiddens.len >= 2:
        let k = forbiddens.len
        for i in 0..k:
            place(forbiddens[i].u, forbiddens[(i + 1) % k].loc)

    var free = 0
    for u in by_depth[d] if forbidden_loc[u] == none:
        while last_depth[Slot(free)] == some(d): free += 1
        place(u, Slot(free))

print(ans)
for group in resp:
    print(group.len, group)
