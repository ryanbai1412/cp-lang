// expect: ok, output "304"
// (spec) R4: a value argument the call also writes is copied in (conformance core-05/param-unchanged-by-a-callee-writing-its-source).
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
    apply(xs, grow)
print(go())
