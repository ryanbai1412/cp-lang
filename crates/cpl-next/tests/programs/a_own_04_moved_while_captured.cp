// expect: E0607
// rule: R05.10.2
// R05.10.2: a captured variable can't be moved while the fn value is alive.
fn go() -> Int:
    var xs = [1, 2]
    let n = () => xs.len
    let ys = xs  // <- here
    n() + ys.len
print(go())
