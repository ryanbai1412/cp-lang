// Source: Yet Another Contest 7 P3 (No More Math Homework), https://dmoj.ca/problem/yac7p3
// Problem: a functional graph P on n <= 1e6 subjects; students i and j must share a group if
//   P^d(i) = P^d(j) for some d. Set one P_A = B to maximise the number of groups; print A B
//   and a group label for each student.
// Solution: a component with cycle length L allows exactly L groups, so the answer is the sum
//   of cycle lengths. Repoint the cycle predecessor of the deepest vertex's cycle root at
//   that vertex: the cycle gains its whole tail. Recompute the cycles and label each vertex
//   by (cycle position + depth) mod L plus the component's offset. O(n).
// original: corpus/submissions/5705218.cpp

let n: 1..=1_000_000 = read()
domain V = 1..=n
var par: [V]V = read()

var rev: [V]List<V> = []
var vis: [V]Int = 0
var on_cyc: [V]Bool = false
var st: List<V> = []
var cyc_nodes: List<V> = []
var cyc_starts: List<Int> = []

fn dfs_cyc(u: V):
    if vis[u] != 0:
        if vis[u] == 1:
            var pos = 0
            while pos < st.len and st[pos] != u:
                pos += 1
            cyc_starts.push(cyc_nodes.len)
            for i in pos..st.len:
                let v = st[i]
                on_cyc[v] = true
                cyc_nodes.push(v)
        return
    vis[u] = 1
    st.push(u)
    for i in 0..rev[u].len:
        dfs_cyc(rev[u][i])
    vis[u] = 2
    st.pop()

fn run_cycles():
    for v in V:
        rev[v] = []
        vis[v] = 0
        on_cyc[v] = false
    cyc_nodes = []
    cyc_starts = []
    for v in V:
        rev[par[v]].push(v)
    for v in V:
        dfs_cyc(v)
    cyc_starts.push(cyc_nodes.len)

run_cycles()

var depth: [V]Int = -1
var anchor: [V]V = V.first!

fn dfs_depth(u: V, root: V, d: Int):
    depth[u] = d
    anchor[u] = root
    for i in 0..rev[u].len:
        let v = rev[u][i]
        if not on_cyc[v]:
            dfs_depth(v, root, d + 1)

for v in V:
    if on_cyc[v]:
        dfs_depth(v, v, -1)

var best_d = 0
var u_nv = V.first!
for v in V:
    if depth[v] > best_d or (depth[v] == best_d and v.idx > u_nv.idx):
        best_d = depth[v]
        u_nv = v

let target_root = anchor[u_nv]
var u_ind = V.first!
for v in V:
    if on_cyc[v] and par[v] == target_root:
        u_ind = v

par[u_ind] = u_nv
run_cycles()

var ans: [V]Int = 0
var offset = 0

fn dfs_label(u: V, d: Int, m: Int):
    ans[u] = offset + d + 1
    for i in 0..rev[u].len:
        let v = rev[u][i]
        if not on_cyc[v]:
            dfs_label(v, (d + 1) % m, m)

let ncycs = cyc_starts.len - 1
for ci in 0..ncycs:
    let lo = cyc_starts[ci]
    let hi = cyc_starts[ci + 1]
    let clen = hi - lo
    for i in 0..clen:
        dfs_label(cyc_nodes[lo + i], i, clen)
    offset += clen

print(u_ind, u_nv)
print(ans)
