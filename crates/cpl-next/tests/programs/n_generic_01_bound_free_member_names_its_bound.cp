// expect: E0606
// rule: R08.2.2
// A generic body is checked against its bounds alone: `T.MIN` on a `T` with no bound giving
// `MIN` is an error that names the bound to write, not an accept checked per instance.
fn lo<T>() -> T = T.MIN  // <- here
print(lo<Int>())
