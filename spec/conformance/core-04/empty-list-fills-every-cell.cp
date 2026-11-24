// expect: ok, output "2 0 1"
// An empty `List()` given to an array of lists is one cell's value, so every cell starts empty (122c592fb).
domain X = 0..3
var adj: [X]List<X> = List()
adj[X(0)].push(X(1))
adj[X(0)].push(X(2))
adj[X(2)].push(X(0))
print(adj[X(0)].len, adj[X(1)].len, adj[X(2)].len)
