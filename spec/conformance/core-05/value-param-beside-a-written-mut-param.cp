// expect: ok, output "1 1 9"
// A call writing one `mut` parameter leaves a value parameter with equal contents alone.
fn w(mut ys: List<Int>):
    ys[0] = 9
fn f(xs: List<Int>, mut ys: List<Int>) -> Int:
    w(mut ys)
    return xs[0]
var a = [1, 2, 3]
var b = [1, 2, 3]
print(f(a, mut b), a[0], b[0])
