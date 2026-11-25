// expect: E0602
// rule: R05 R7
// R7 inside a lambda: xs is written while a view of it is still used.
fn go() -> Int:
    let f = () =>
        var xs = [[1], [2]]
        let row = xs[0]
        xs.push([3])  // <- here
        row.len
    f()
print(go())
