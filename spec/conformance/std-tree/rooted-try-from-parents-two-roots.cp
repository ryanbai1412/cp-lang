// expect: ok, output "YES"
domain V = 0..3
let par: [V]?V = [None, None, Some(V(0))]
print(yes(Rooted.try_from_parents(par) == None))
