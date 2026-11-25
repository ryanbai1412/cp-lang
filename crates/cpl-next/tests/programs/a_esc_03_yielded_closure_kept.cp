// expect: E0603
// rule: R05.10.2, R8
// A generator yields a closure over its own var; the loop body keeps it past the loop.
fn gen() -> yields () -> Int:
    var c = 0
    yield () => { c += 1; c }
fn go() -> Int:
    var keep = () => 0
    for f in gen():
        keep = f  // <- here
    keep()
print(go())
