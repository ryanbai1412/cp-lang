// expect: E0607
// rule: R05.10.2
// moved while captured, inside a lambda.
fn go() -> Int:
    let f = () =>
        var xs = [1, 2]
        let n = () => xs.len
        let ys = xs  // <- here
        n() + ys.len
    f()
print(go())
