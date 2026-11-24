// expect: E0210
// rule: 12.14.1
// An `@intrinsic` body names an entry of the intrinsics table.
fn f(x: Int) -> Int = @intrinsic("no_such") // <- here
print(f(1))
