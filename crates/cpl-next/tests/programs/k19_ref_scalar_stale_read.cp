// expect: ok, output "1099"
// Scalar var passed by value, closure writes it during the call: callee must keep its own value.
fn h(a: Int, f: () -> ()) -> Int:
    f()
    a
fn go() -> Int:
    var v = 1
    let w = () => v = 99
    h(v, w) * 1000 + v
print(go())
