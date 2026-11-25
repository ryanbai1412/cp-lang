// expect: E0603
// rule: R05.10.2, R8
// R8: a closure that borrows a local var is stored in a top-level var, which outlives it.
var keep = () => 0
fn arm():
    var c = 41
    keep = () => { c += 1; c }  // <- here
arm()
print(keep())
