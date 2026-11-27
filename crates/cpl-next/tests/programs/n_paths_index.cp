// expect: ok, output "0 2 -1\n2"
// rule: R12.4.5
// `p[v]` on a search's `Paths` reads through std's ordinary `IndexGet` impl, so a generic fn
// bound by `IndexGet` reads it the same way.
use std.seq.{IndexGet}
domain V = 0..4
let g: [V]List<V> = [[V(1)], [V(2)], [], []]
let p = bfs(g, from: [V(0)])
print(p[V(0)] ?? -1, p[V(2)] ?? -1, p[V(3)] ?? -1)
fn far<S: IndexGet<V, ?Int>>(s: S) -> Int:
    var m = 0
    for v in V: m = max(m, s[v] ?? 0)
    m
print(far(p))
