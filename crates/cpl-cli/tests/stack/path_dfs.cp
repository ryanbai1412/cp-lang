// A depth-first search down a path of 10^6 vertices: 10^6 calls deep.
let n: 1..=1e6 = read()
domain V = 0..n
var seen: [V]Bool = false
var order: [V]Int = 0
var t = 0
fn dfs(v: V):
    seen[v] = true
    order[v] = t
    t += 1
    if v.ord + 1 < n:
        let w = V.at(v.ord + 1)
        if not seen[w]: dfs(w)
dfs(V.at(0))
print(t, order[V.at(n - 1)])
