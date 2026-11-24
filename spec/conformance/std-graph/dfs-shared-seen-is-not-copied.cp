// expect: ok, output "200000"
// rule: std.graph.dfs
// `dfs(seen: mut s)` reads and writes `s` in place, so one walk per vertex over a shared `seen` is O(n + m) in total, not O(n) per walk.
domain V = 0..200000
let g: [V]List<V> = [if v.idx + 1 < 200000: [V(v.idx + 1)] else: [] for v in V]
var seen: [V]Bool = false
var walks = 0
for v in V.rev:
    for e in dfs(g, from: [v], seen: mut seen):
        if let Visit.Enter(_, _) = e: walks += 1
print(walks)
