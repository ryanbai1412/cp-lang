// expect: E0303 at 6
// rule: 06.4.3
// A `@check` fn reads the input the file reads, so it takes no parameters; the error names `@check` and points at the fn.
let n: 1..=5 = read()
@check
fn c(ans: Int) -> Bool = ans > 0
print(n)
