// expect: E0210
// rule: R12.14.2
// An `@intrinsic` fn takes its entry's parameters as well as giving its result: `sqrt` takes one
// `Int` or `F64`, so a second parameter is not that signature, while a type parameter stands
// for the entry's types.
fn root<N>(x: N) -> F64 = @intrinsic("sqrt")
fn both(x: F64, y: F64) -> F64 = @intrinsic("sqrt") // <- here
print(root(4.0) + both(1.0, 2.0))
