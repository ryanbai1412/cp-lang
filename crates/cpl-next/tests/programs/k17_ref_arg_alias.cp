// expect: E0602
// (spec) R10
// The closure writes a var that is also passed by value to the fn that calls the closure.
fn apply(xs: List<Int>, f: () -> ()) -> Int:
    let a = xs.len
    f()
    f()
    f()
    f()
    f()
    a * 100 + xs.len + xs[0]
fn go() -> Int:
    var xs: List<Int> = [1, 2, 3]
    let grow = () => xs.push(7)
    apply(xs, grow)  // <- here
print(go())
