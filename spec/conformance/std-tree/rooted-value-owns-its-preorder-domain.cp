// expect: ok, output "YES 3 2 4"
// rule: std.tree.Rooted.new
// `let t = Rooted.new(..)` owns its preorder domain `t.Pre`: `t.pos(v)` is a `t.Pre`, a `[t.Pre]` table reads `t.at`, and `t.down`'s callback takes `(T, V)` with nothing written.
use std.tree.{Rooted}
let n = 4
domain V = 0..n
let w: [V]Int = [V] v => (v.ord * 3) % 5
let g: [V]List<V> = [[V(1), V(2)], [V(0), V(3)], [V(0)], [V(1)]]
let t = Rooted.new(g, V(0))
fn earlier(a: V, b: V) -> Bool:
    t.pos(a) < t.pos(b)
let top = t.down(V(0), (r, v) => if w[v] >= w[r]: v else: r)
let by_pre = [t.Pre] p => w[t.at[p]]
print(yes(earlier(V(0), V(3))), top[V(3)], top[V(2)], by_pre[t.pos(V(3))])
