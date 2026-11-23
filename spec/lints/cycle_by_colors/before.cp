let (n, m): (1..=100000, 0..=200000) = read()
var out: List<List<Int>> = [[] for _ in 0..=n]
for _ in 0..m:
    let (u, v): (1..=n, 1..=n) = read()
    out[u].push(v)
var color: List<Int> = [0 for _ in 0..=n]
var cyc = false
fn dfs(u: Int):
    color[u] = 1
    for v in out[u]:
        if color[v] == 1:
            cyc = true
        elif color[v] == 0:
            dfs(v)
    color[u] = 2
for u in 1..=n:
    if color[u] == 0:
        dfs(u)
print(yes(cyc))
