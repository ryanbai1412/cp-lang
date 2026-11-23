let (n, m): (1..=200000, 0..=200000) = read()
var adj: List<List<Int>> = [[] for _ in 0..=n]  // expect: manual_adjacency_list
for _ in 0..m:
    let (u, v): (1..=n, 1..=n) = read()
    adj[u].push(v)
    adj[v].push(u)  // expect: symmetric_write
var deg_max = 0
for u in 1..=n:
    deg_max = max(deg_max, adj[u].len)  // expect: chmin_chmax_if|loop_reduction
print(deg_max)
