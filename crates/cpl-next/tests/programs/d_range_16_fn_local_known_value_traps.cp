// expect: R0201
// rule: R04.4.24
// A local is not a constant, whatever it is known to hold: storing it out of range is the
// checked narrowing, not E0902.
fn half(x: 0..=100) -> Int = x / 2
fn g() -> Int:
    let n = 200
    half(n)
print(g())
