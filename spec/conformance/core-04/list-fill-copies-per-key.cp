// expect: ok, output "1\n0\n0"
// rule: 04.4.27
// A fill takes any value, not only a scalar: `= List()` gives every key its own empty list, so a push to one leaves the others empty (rebuild round 32 FL11).
domain X = 1..=3
var adj: [X]List<X> = List()
adj[X(1)].push(X(2))
for x in X: print(adj[x].len)
