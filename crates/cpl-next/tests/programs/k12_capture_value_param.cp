// expect: ok, output "7 5"
// A by-value parameter captured and reassigned through the closure: the caller's variable is untouched.
fn f(n: Int) -> Int:
    var m = n
    let bump = () => m += 1
    bump()
    bump()
    m
var g = 5
print(f(g), g)
