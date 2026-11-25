// expect: E0602
// rule: R05 section 9, R10
// Same, but the push is hidden in a closure called by the body.
fn go() -> Int:
    var xs = [3, 1, 2]
    let grow = () => xs.push(0)
    for x in xs: grow()  // <- here
    xs.len
print(go())
