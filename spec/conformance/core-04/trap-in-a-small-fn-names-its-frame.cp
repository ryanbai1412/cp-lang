// expect: R0203
// input: "0"
// has: call stack: f
// A trap inside a small function names that function in its call stack, even when the function is short enough to run inlined.
fn f(a: Int, b: Int) -> Int:
    var r = 0
    r = a / b // <- here
    return r
let n: Int = read()
print(f(10, n))
