// expect: R0203
// input: "1"
// has: call stack: f (line 8) <- top level (line 16)
// A trap in a fn reached after a branch joins reports the call's line, whichever way led
// there: the engine reads lines from a table, not from ops that set them as it runs.
fn f(a: Int, b: Int) -> Int:
    var r = 0
    r = a / b  // <- here
    return r
let n: Int = read()
var d = 0
if n > 0:
    d = n - 1
else:
    d = n + 1
print(f(10, d))
