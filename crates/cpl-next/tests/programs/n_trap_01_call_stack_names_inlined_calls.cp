// expect: R0203
// input: "0"
// has: "call stack: g (line 7) <- f (line 10) <- top level (line 12)"
// A trap two small calls deep names both calls, even when they run inlined.
fn g(a: Int, b: Int) -> Int:
    var r = 0
    r = a / b // <- here
    return r
fn f(b: Int) -> Int:
    return g(10, b) + 1
let n: Int = read()
print(f(n))
