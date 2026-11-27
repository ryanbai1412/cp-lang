// expect: ok, output "2 1\n3 1 2"
// rule: R05.8.2
// `swap(mut a, mut b)` is an ordinary core fn over a tuple assignment, for any type.
var i = 1
var j = 2
swap(mut i, mut j)
print(i, j)
var xs = [1, 2]
var ys = [3]
swap(mut xs, mut ys)
print(xs, ys)
