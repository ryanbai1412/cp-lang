// YAC 7 P3 — original: corpus/submissions/5705218.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)

let n: 1..=1_000_000 = read()
domain V = 1..=n
var par: [V]V = read()

enum Mark = Unseen | OnStack | Done

// FRICTION 39.5: Perm<V>.cycles() does not cover general functional graphs [V]V
fn find_cycles(p: [V]V) -> ([V]List<V>, [V]Bool, List<List<V>>):
    var rev: [V]List<V> = [V] _ => []
    for v in V:
        rev[p[v]].push(v)
    var mark: [V]Mark = Unseen
    var on_cyc: [V]Bool = false
    var st: List<V> = []
    var cycs: List<List<V>> = []

    fn dfs(u: V):
        match mark[u]:
            Done => return
            OnStack =>
                let start = st.find(u)!
                let cyc = [st[i] for i in start..st.len]
                for v in cyc: on_cyc[v] = true
                cycs.push(cyc)
                return
            Unseen => ()
        mark[u] = OnStack
        st.push(u)
        for v in rev[u]:
            dfs(v)
        st.pop()
        mark[u] = Done

    for v in V:
        dfs(v)
    (rev, on_cyc, cycs)

let (rev1, on_cyc1, _) = find_cycles(par)
var depth: [V]Int = -1
var anchor: [V]V = [V] v => v

fn dfs_depth(u: V, root: V, d: Int):
    depth[u] = d
    anchor[u] = root
    for v in rev1[u] if not on_cyc1[v]:
        dfs_depth(v, root, d + 1)

for v in V if on_cyc1[v]:
    dfs_depth(v, v, -1)

let u_nv = max_by((v for v in V if depth[v] >= 0), v => (depth[v], v)) ?? V.first
let target_root = anchor[u_nv]
let u_ind = first(v for v in V if on_cyc1[v] and par[v] == target_root)!
par[u_ind] = u_nv

let (rev2, on_cyc2, cycs) = find_cycles(par)
var ans: [V]Int = 0
var offset = 0

fn label(u: V, d: Int, m: Int):
    ans[u] = offset + d + 1
    for v in rev2[u] if not on_cyc2[v]:
        label(v, (d + 1) % m, m)

for cyc in cycs:
    for (i, v) in cyc.items:
        label(v, i, cyc.len)
    offset += cyc.len

print(u_ind, u_nv)
print(" ".join("{ans[v]}" for v in V))
