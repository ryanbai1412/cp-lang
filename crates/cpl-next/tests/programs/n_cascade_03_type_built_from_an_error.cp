// expect: E0204
// rule: diagnostics
// lacks: "expected"
// lacks: "does not implement"
// lacks: "slice:"
// A type built from an unknown one, `List<Nope>`, meets no verdict of its own: passing it on, or
// asking it for an impl, is downstream of the one unknown type.
fn total(xs: List<Int>) -> Int = xs.len
let xs: List<Nope> = [] // <- here
print(total(xs), if xs == xs: 1 else: 0)
