// expect: ok, output "12 3\n3 1\n6\n9"
// A function type keeps its `mut` parameters: a fn value or lambda that takes `mut s` is called
// `f(mut x)`, and its writes reach the caller's place (R01.8.1, R05.4.1).
fn apply(mut x: Int, y: Int, f: (mut Int, Int) -> ()):
    f(mut x, y)
fn add(mut s: Int, d: Int):
    s += d
var a = 10
apply(mut a, 2, add)
var b = 7
apply(mut b, 3, (mut s, d) => s = d)
print(a, b)
let swap_ends: (mut List<Int>) -> () = (mut xs) => xs.swap(0, xs.len - 1)
var l = [1, 2, 3]
swap_ends(mut l)
print(l[0], l[2])
var total = 0
let steps: List<(mut Int, Int) -> ()> = [add, (mut s, d) => s += 2 * d]
for g in steps:
    g(mut total, 2)
print(total)
struct Acc(n: Int, bump: (mut Int) -> ())
var acc = Acc(4, (mut s) => s += 5)
acc.bump(mut acc.n)
print(acc.n)
