// expect: E0602
// rule: R05 R7
// Control: the same as c03 outside a lambda.
fn go() -> Int:
    var xs = [[1], [2]]
    let row = xs[0]
    xs.push([3])  // <- here
    row.len
print(go())
