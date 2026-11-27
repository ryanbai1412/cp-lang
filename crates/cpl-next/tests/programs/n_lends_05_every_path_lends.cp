// expect: ok, output "9 2\n1 8"
// Every path through a `lends mut` block body reaches one `lend mut place`, and code after
// it runs once the caller is done (R05.6.3).
fn pick(mut xs: List<Int>, hi: Bool) -> lends mut Int:
    if hi:
        lend mut xs[1]
    else:
        let k = 0
        lend mut xs[k]
var a = [1, 2]
pick(mut a, false) = 9
print(a[0], a[1])
var b = [1, 2]
pick(mut b, true) += 6
print(b[0], b[1])
