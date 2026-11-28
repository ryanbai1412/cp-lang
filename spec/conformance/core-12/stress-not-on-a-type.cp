// expect: E0325
// rule: 12.13.3
// `@stress(kind)` says how stress makes a type alias's values; a fn has none to make (decision 25.32).
@stress(tree) // <- here
fn f() -> Int: 1
print(f())
