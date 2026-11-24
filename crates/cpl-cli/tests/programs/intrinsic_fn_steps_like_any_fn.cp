// A traced run of an `@intrinsic` fn shows its call's result as for any
// other fn: the table's primitive runs the same traced as untraced.
fn ones(x: Int) -> Int = @intrinsic("popcount")
let a = ones(7)
let f = ones
let b = f(255)
print(a, b, ones(-1))
