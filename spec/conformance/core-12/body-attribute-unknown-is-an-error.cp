// expect: E0102
// rule: 12.14.1
// A body attribute other than `@cpp` and `@intrinsic` is an error, never an empty body: a compiler older than its std fails loudly.
fn ones(x: Int) -> Int = @intrinsics("popcount") // <- here
print(ones(7))
