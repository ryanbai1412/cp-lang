// expect: ok, output "YES YES YES\nYES YES\nYES"
use std.pair.{Pair}
domain V = 0..4
let p = Pair<V>.new(V(3), V(1))
print(yes(p.lo == V(1)), yes(p.hi == V(3)), yes(p.other(V(3)) == V(1)))
print(yes(Pair<V>.at(V(2), V(2)) == None), yes(Pair<V>.at(V(2), V(0)) == Some(Pair<V>.new(V(0), V(2)))))
print(yes(Pair<V>.new(V(3), V(1)) == Pair<V>.new(V(1), V(3))))
