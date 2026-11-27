// expect: ok, output "a 1\na\n2 3\n3 2"
// rule: R02.1.2
// A variadic call lends the values it packs, like any argument: they are still there after.
fn f<T: Show>(..xs: T):
    print(xs)
let t = "a"
f(t, 1)
print(t)
fn g():
    var ys = [3]
    print(2, ys)
    print(ys, 2)
g()
