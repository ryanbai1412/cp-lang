// expect: E0211
// Working out a `const` has a fixed step budget; one that needs more is an error.
fn spin(n: Int) -> Int:
    var s = 0
    for i in 0..n: s += i % 7
    return s
const X = spin(100_000_000)  // <- here
print(X)
