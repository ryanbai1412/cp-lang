// expect: E0204
// rule: diagnostics
// lacks: "slice:"
// A value whose type is built from an error prints as the error: nothing more is said of it.
let g: ?Nope = None // <- here
print(g, 1)
