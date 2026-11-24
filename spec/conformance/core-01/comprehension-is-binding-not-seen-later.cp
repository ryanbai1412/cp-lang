// expect: E0201
// has: "if let"
// An `is` pattern binds only inside its own test, so the element can't see `v`; the fix binds it with `if let Some(v) = x` (decision 111).
let xs = [Some(1), None, Some(3)]
print([v for x in xs if x is Some(v)]) // <- here
