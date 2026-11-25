// expect: ok, output "YES YES YES YES"
// rule: R04.2
// Positions are `Eq` and `Ord` through core's impls for every domain, so optionals, lists
// and tuples of them compare too.
domain V = 0..3
let a = V.at(1)
print(yes(a == Some(V(1))), yes(a != None), yes([V(0), V(2)] == [V(0), V(2)]), yes((V(1), 2) < (V(2), 0)))
