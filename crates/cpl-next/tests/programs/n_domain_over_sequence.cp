// expect: ok, output "3 4 10 5 4 3\n3"
// rule: R04.2.29
// `domain V = xs` has one position per element of a list, string or array, and another
// domain's value has that domain's count: each through its impl of the `@role(counted)` trait.
domain V = [4, 5, 6]
var s = 0
for v in V: s += 1
let xs = [7, 8, 9, 10]
domain W = xs
let w: [W]Int = [W] k => xs[Int(k)]
domain C = "hello"
domain A = w
domain B = V
print(s, W.count, w[W(3)], C.count, A.count, B.count)
fn inner(ys: List<Int>) -> Int:
    domain Y = ys
    var t = 0
    for y in Y: t += Int(y)
    t
print(inner([1, 1, 1]))
