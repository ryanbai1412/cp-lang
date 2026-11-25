// expect: E0603
// rule: R05.7.8
// A lent closure returned through a second closure that calls it (captures the closure by value).
fn make() -> () -> Int:
    var c = 0
    let f = () => { c += 1; c }
    let g = () => f() + 100
    g  // <- here
let h = make()
print(h(), h())
