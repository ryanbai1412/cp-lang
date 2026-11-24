// expect: ok, output "2 0 1"
// `seen:` is read and updated in place, so a second walk over it finds nothing.
domain V = 0..3
let g: [V]List<V> = [[1], [], []]
var seen: [V]Bool = false
let n1 = dfs(g, from: [V(0)], seen: mut seen).count()
let n2 = dfs(g, from: [V(0)], seen: mut seen).count()
print(n1 / 2, n2, Int(seen[V(1)]))
