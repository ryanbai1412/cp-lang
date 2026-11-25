// expect: E0602
// rule: R05 section 9
// for x in xs inside a lambda body, pushing to xs.
fn go() -> Int:
    let f = () =>
        var xs = [3, 1, 2]
        for x in xs: xs.push(x)  // <- here
        xs.len
    f()
print(go())
