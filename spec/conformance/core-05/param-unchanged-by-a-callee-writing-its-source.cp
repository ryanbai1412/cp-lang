// expect: ok, output "4 9"
// A callee that writes the global a value parameter came from keeps that parameter from being lent: it still reads the value at the call.
var a = [1, 2, 3]
fn w():
    a[0] = 9
fn f(xs: List<Int>) -> Int:
    w()
    return xs[0] + xs.len
print(f(a), a[0])
