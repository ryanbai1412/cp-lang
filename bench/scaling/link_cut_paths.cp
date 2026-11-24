// n: 2048
use std.rand.{Rng}
use std.tree.{PreferredPaths}
let n: Int = read()
domain V = 0..n
var rng = Rng(3)
let par: [V]?V = [V] v => if v.ord == 0: None else: Some(V(rng.int(0, v.ord - 1)))
domain W = 0..32 * n
var wpar: [W]?W = [W] w => if w.ord == 0: None else: Some(W(rng.int(0, w.ord - 1)))
let wt = Rooted.from_parents(wpar)
var s = 0
for _ in 0..40:
    var pp0 = PreferredPaths(wt) // timed: tree.PreferredPaths.new
    s += pp0.hd[W(0)].ord
let t = Rooted.from_parents(par)
var pp = PreferredPaths(t)
for i in 0..32 * n:
    for x in pp.access(V((i * 7919) % n)): s += 1 // timed: tree.PreferredPaths.access
print(s)
