// expect: ok, output "0 1 -1"
domain V = 0..2
let par: [V]?V = [None, Some(V(0))]
let t = Rooted.from_parents(par)
print(t.depth(V(0)), t.depth(V(1)), t.parent(V(0)).map(v => Int(v)) ?? -1)
