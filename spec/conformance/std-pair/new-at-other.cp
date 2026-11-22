// expect: ok, output "YES YES YES\nYES YES\nYES"
use std.pair.{Pair}
domain V = 0..4
let p = Pair<V>.new(V(3), V(1))
print(p.lo == V(1), p.hi == V(3), p.other(V(3)) == V(1))
print(Pair<V>.at(V(2), V(2)) == None, Pair<V>.at(V(2), V(0)) == Some(Pair<V>.new(V(0), V(2))))
print(Pair<V>.new(V(3), V(1)) == Pair<V>.new(V(1), V(3)))
