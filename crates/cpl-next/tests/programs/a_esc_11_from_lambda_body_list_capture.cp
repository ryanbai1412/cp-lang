// expect: E0603
// rule: R05.10.2, R8
// Same with a list local to the lambda captured by reference.
fn go() -> Int:
    var keep = () => 0
    let f = () =>
        var xs = [1, 2, 3]
        keep = () => { xs.push(4); xs.len }  // <- here
    f()
    keep() + keep()
print(go())
