// expect: ok, output "YES"
// reachable yields dfs's Enter order from any sources, touching only what it reaches.
var ok = true
for trial in 0..40:
    domain V = 0..30
    let g: [V]List<V> = [V] _ => [V(rand(0, 29)) for _ in 0..rand(0, 3)]
    let from = [V(rand(0, 29)) for _ in 0..rand(0, 3)]
    var want: List<V> = []
    for e in dfs(g, from):
        if let Visit.Enter(v, _) = e: want.push(v)
    if reachable(g, from).collect() != want: ok = false
print(yes(ok))
