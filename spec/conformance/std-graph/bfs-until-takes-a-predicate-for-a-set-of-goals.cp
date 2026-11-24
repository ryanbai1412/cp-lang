// expect: ok, output "YES"
// `until:` is a predicate: a search stops at the first settled vertex it holds for, so a set
// of goals needs no single target, and `v => v == t` is the one-target stop.
domain V = 0..10
let g = Graph.directed([(V(0), V(1)), (V(1), V(2)), (V(2), V(3)), (V(0), V(4)), (V(4), V(5)), (V(5), V(6))])
let all = bfs(g, from: [V(0)])
let some = bfs(g, from: [V(0)], until: v => v.ord % 3 == 2 and v.ord > 2)
let one = bfs(g, from: [V(0)], until: v => v == V(2))
var ok = all[V(6)]! == 3 and all[V(3)]! == 3
ok = ok and some[V(5)]! == 2 and (some[V(6)] ?? -1) == -1 and (some[V(3)] ?? -1) == -1
ok = ok and one[V(2)]! == 2 and (one[V(3)] ?? -1) == -1 and (one[V(6)] ?? -1) == -1
print(if ok: "YES" else: "NO")
