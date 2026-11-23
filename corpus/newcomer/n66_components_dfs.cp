let (n, m): (1..=100000, 0..=200000) = read()
var adj: List<List<Int>> = [[] for _ in 0..=n]  // expect: missing adjacency_build_loop
for _ in 0..m:
    let (u, v): (1..=n, 1..=n) = read()
    adj[u].push(v)
    adj[v].push(u)  // expect: symmetric_write
var seen: List<Bool> = [false for _ in 0..=n]
fn visit(u: Int):
    seen[u] = true
    for v in adj[u]:
        if not seen[v]:
            visit(v)
var comps = 0
for u in 1..=n:
    if not seen[u]:  // expect: missing components_by_dfs
        comps += 1
        visit(u)
print(comps)
