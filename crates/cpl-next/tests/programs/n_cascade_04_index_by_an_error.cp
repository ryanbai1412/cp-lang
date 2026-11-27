// expect: E0201
// rule: diagnostics
// lacks: "does not implement"
// lacks: "slice:"
// Indexing by an expression already diagnosed is that error again: the slice it would be has no
// type to call `.sum()` on.
let xs = [1, 2, 3]
let w = xs[nope] // <- here
print(w.sum())
