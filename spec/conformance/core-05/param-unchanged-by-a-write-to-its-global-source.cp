// expect: ok, output "1 9"
// A value parameter is its own value: the callee writing the global it was passed from does not change what the parameter reads, even when the compiler lends it.
var a = [1, 2, 3]
fn f(xs: List<Int>) -> Int:
    a[0] = 9
    return xs[0]
print(f(a), a[0])
