// expect: E0204
// rule: diagnostics
// lacks: "is not indexed"
// lacks: "slice:"
// A type built from an unknown one has no impl to look for: indexing it is downstream of the one
// unknown type.
struct Bag<T>(x: T)
let b: Bag<Nope> = Bag(1) // <- here
print(b[0] + 1)
