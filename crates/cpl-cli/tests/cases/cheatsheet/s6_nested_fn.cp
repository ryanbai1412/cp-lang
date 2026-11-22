// cp: 0.1 bea787c+dirty
let n: 1..=10 = read()
domain V = 1..=n
let a: [V]Int = read()
let v = V(2)
var vis: [V]Bool = false
fn dfs(u: V):
    vis[u] = true
    if let w = u + 1:
        if not vis[w]: dfs(w)
dfs(V(2))
print(vis)
