// expect: E0603
// rule: R05.10.2, R8
// R8 inside a lambda body: lent closure stored in the enclosing fn's var, used after the lambda returns.
fn go() -> Int:
    var keep = () => 0
    let f = () =>
        var c = 41
        keep = () => { c += 1; c }  // <- here
    f()
    let pad = [1, 2, 3, 4, 5, 6, 7, 8]
    keep() + pad.len
print(go())
