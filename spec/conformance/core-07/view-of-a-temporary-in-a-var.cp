// expect: E0603
// rule: 07.5.10 05.7.8
// A `var` keeping a view of an unnamed temporary outlives its source, exactly like a `let` (R07.5.10).
var w = [3, 1, 2].sorted()[0..1] // <- here
print(w.sum())
