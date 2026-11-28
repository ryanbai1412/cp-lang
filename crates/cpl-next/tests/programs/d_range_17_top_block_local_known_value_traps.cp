// expect: R0201
// rule: R04.4.24
// A binding inside a top-level block is not a module-level `let`, so not a constant either.
fn half(x: 0..=100) -> Int = x / 2
if true:
    let n = 200
    print(half(n))
