// expect: ok, output "YES NO"
use std.dsu.{DsuOf}
domain V = 0..3
var d = DsuOf<V, Int>(init: v => Int(v) + 1, join: (x, y) => x + y)
d.unite(V(0), V(1))
let m = d.mark()
d.unite(V(1), V(2))
d.undo_to(m)
print(yes(d.same(V(0), V(1))), yes(d.same(V(1), V(2))))
