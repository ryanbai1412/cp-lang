// expect: ok, output "1 3 YES\nYES YES\n1 3"
use std.pair.{Pair, symmetric}
domain V = 0..4
let p = Pair<V>(V(3), V(1))
print(Int(p.lo), Int(p.hi), p == Pair<V>(V(1), V(3)))
print(Pair<V>.at(V(2), V(2)) == None, Pair<V>.at(V(2), V(0)) == Some(Pair<V>(V(0), V(2))))
print(Int(p.other(V(3))), Int(p.other(V(1))))
