// expect: E0204
// rule: diagnostics
// lacks: "expected"
// lacks: "slice:"
// A cell of a place whose type is built from an error is that error: its index and value are not
// fitted to it.
var m: List<Nope> = [] // <- here
m[true] = 1
m[true] += 2
