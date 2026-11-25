// expect: E0603
// rule: R05.7.8
// R05.7.8 inside a lambda: the lambda returns a view of its own local.
fn go() -> Int:
    let f = () =>
        var xs = [[1], [2]]
        let g = () => xs.len  // captures xs by reference
        g  // <- here
    let h = f()
    h()
print(go())
