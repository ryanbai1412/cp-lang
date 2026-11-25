// expect: E0608
// (spec) R2
// Explicit copy, then write the original while walking the copy: legal, cost is visible.
fn go() -> Int:
    var xs: List<Int> = [1, 2, 3]
    let snap = xs
    for x in snap: xs.push(x)  // <- here
    xs.len
print(go())
