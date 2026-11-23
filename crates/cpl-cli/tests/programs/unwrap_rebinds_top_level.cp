// A top-level `let at = at!` ends the fill phase (R02.4.8): later lines and
// fns read the unwrapped `at`, the loop above it filled the optional one.
let n = 4
var at: [0..n]?Int = None
for i in 0..n: at[i] = i * 3
let at = at!
fn last() -> Int = at[n - 1]
print(at[0] + at[1], last())
