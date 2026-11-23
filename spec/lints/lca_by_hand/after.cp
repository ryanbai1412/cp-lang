domain V = 0..4
let par: [V]?V = [None, Some(0), Some(0), Some(1)]
let t = Rooted.from_parents(par)
print(t.lca(3, 2))
