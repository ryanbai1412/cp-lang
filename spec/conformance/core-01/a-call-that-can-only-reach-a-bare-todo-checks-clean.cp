// expect: R0402
// rule: 01.6.8
// A bare `todo` is a placeholder for code not written yet: a call that can only reach it still checks clean, and stops when it runs.
struct Op(f: Int -> Int)
fn missing(x: Int) -> Int = todo // <- here
let o = Op(missing)
print(o.f(1))
