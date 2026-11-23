let (n, m): (1..=200000, 0..=200000) = read()
var adj: List<List<Int>> = [[] for _ in 0..=n]
for _ in 0..m:
    let (u, v): (1..=n, 1..=n) = read()
    adj[u].push(v)
print(adj[1].len)
