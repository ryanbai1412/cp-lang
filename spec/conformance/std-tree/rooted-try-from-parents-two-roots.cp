// expect: ok, output "YES"
// Two roots make a forest (`from_parents` docs); this case once expected `None` here, so it now
// checks the forest is accepted and `rooted-try-from-parents-cycle` covers the `None` case.
domain V = 0..3
let par: [V]?V = [None, None, Some(V(0))]
print(yes(Rooted.try_from_parents(par) != None))
