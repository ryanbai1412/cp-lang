// expect: E0603
// rule: R05.10.2, R8
// R8 inside a lambda body: a closure that borrows the lambda's local var is stored in a top-level var.
var keep = () => 0
fn arm():
    let f = () =>
        var c = 41
        keep = () => { c += 1; c }  // <- here
    f()
arm()
print(keep())
