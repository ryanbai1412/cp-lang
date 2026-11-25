// expect: E0602
// rule: R05 section 9 example
// for x in xs: xs.push(x) is read-lent to the loop (spec ch5 section 9 example).
fn go() -> Int:
    var xs = [3, 1, 2]
    for x in xs: xs.push(x)  // <- here
    xs.len
print(go())
