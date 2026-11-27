// expect: E0303
// rule: diagnostics
// lacks: "has no member"
// lacks: "expected `Int`"
// A call its arguments do not fit is diagnosed there; its value is the error, so its uses say
// nothing more.
fn f(x: Int) -> Int = x
let y = f(1, 2) // <- here
print(y.nope)
let z: Str = f(1, 2)
