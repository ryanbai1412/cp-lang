// expect: E0602
// (spec) R10
// The var is lent `mut` to a callee that also reaches it through a closure.
fn app(mut xs: List<Int>, f: () -> ()):
    xs.push(1)
    f()
    xs.push(2)
fn go() -> Int:
    var xs: List<Int> = []
    let clear = () => xs = []
    app(mut xs, clear)  // <- here
    xs.len
print(go())
