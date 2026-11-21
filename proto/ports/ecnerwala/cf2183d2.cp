// Codeforces 2183 D2: compute the minimum number of operations by depth, then assign
// vertices bottom-up into slots so no parent shares a slot with its child.
@!multitest
let n: Int = read()
domain V = 1..=n
domain E = 0..n - 1
let edges: [E](u: V, v: V) = read()
let g = Graph.undirected(edges)

var by_depth: [0..n]List<(u: V, p: ?V)> = []

fn dfs(u: V, p: ?V, depth: Int):
    by_depth[depth].push((u, p))
    for a in g.out(u):
        if p == a.to: continue
        dfs(a.to, u, depth + 1)

dfs(V(1), none, 0)

var ans = 0
for d in 0..n:
    if by_depth[d].is_empty: break
    let sz = by_depth[d].len
    let p0 = by_depth[d][0].p
    let same_parent = d > 0 and all(p == p0 for (_, p) in by_depth[d])
    ans max= sz + Int(same_parent)

var resp: [0..ans]List<V> = []
var last_depth: [0..ans]Int = n + 1
var forbidden_loc: [V]?Int = none

for d in (0..n).rev:
    if by_depth[d].is_empty: continue

    fn place(u: V, p: ?V, loc: Int):
        resp[loc].push(u)
        last_depth[loc] = d
        if let par = p: forbidden_loc[par] = loc

    var forbiddens: List<(u: V, p: ?V, loc: Int)> = []
    for (u, p) in by_depth[d]:
        if let fl = forbidden_loc[u]:
            forbiddens.push((u, p, fl))

    if forbiddens.len == 1:
        let (u, p, _) = forbiddens[0]
        var loc = 0
        while last_depth[loc] == d + 1: loc += 1
        place(u, p, loc)
    elif not forbiddens.is_empty:
        let first_loc = forbiddens[0].loc
        var (prev_u, prev_p) = (forbiddens[0].u, forbiddens[0].p)
        for (u, p, fl) in forbiddens[1..]:
            place(prev_u, prev_p, fl)
            (prev_u, prev_p) = (u, p)
        place(prev_u, prev_p, first_loc)

    var loc = 0
    for (u, p) in by_depth[d]:
        if forbidden_loc[u] == none:
            while last_depth[loc] == d: loc += 1
            place(u, p, loc)

print(ans)
for group in resp:
    print(group.len, group)
