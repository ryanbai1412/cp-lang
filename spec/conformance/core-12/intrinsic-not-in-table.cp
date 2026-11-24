// expect: E0210
// rule: 12.14.5
// The intrinsics table grows only by a decision entry: a program can't add an entry by naming one.
fn ones(x: Int) -> Int = @intrinsic("my_popcount") // <- here
print(ones(11))
