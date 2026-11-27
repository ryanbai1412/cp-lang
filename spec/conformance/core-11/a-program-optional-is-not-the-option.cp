// expect: E0302
// rule: 11.6.1
// A program's own `Optional` is just a struct: `Some(3)` still builds std's `?Int`, the `option`
// role, which does not fit it.
struct Optional(v: Int)
let x: Optional = Some(3) // <- here
print(x.v)
