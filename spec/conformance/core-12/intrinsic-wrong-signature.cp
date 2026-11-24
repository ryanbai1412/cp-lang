// expect: E0210
// rule: 12.14.2
// An `@intrinsic` fn has its entry's signature: `popcount` gives an `Int`, not a `Bool`.
fn f(x: Int) -> Bool = @intrinsic("popcount") // <- here
print(f(1))
