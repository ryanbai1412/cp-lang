// expect: E0107
// rule: 05.1.1
// What the compiler trusts unproved is a closed list of `@` attributes; another is no promise.
@noalias // <- here
fn f(xs: List<Int>) -> Int = xs.len
print(f([1]))
