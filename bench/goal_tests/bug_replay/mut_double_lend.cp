fn f(mut a: List<Int>, mut b: List<Int>):
    a.push(1)
    b.push(2)
var xs: List<Int> = []
f(mut xs, mut xs)
print(xs.len)
