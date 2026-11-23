// expect: ok, output "YES"
// A cycle in the parents is the one way `try_from_parents` gives `None`.
domain V = 0..3
let par: [V]?V = [None, Some(V(2)), Some(V(1))]
print(yes(Rooted.try_from_parents(par) == None))
