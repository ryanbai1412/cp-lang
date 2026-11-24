// expect: ok, output "0 2 4\n1 0 0"
// rule: 02.8.3
let n = 3
let d: [n]Int = [i * 2 for i in 0..n]
var adj: [n]List<Int> = [[] for _ in 0..n]
adj[0].push(1)
print(d)
print(adj[0].len, adj[1].len, adj[2].len)
