// expect: E0325
// `@stress` takes one bare kind of perm, tree, simple, connected, dag and digraph (decision 25.32).
@stress(forest) // <- here
type T = [0..2](u: 1..=3, v: 1..=3)
print(1)
