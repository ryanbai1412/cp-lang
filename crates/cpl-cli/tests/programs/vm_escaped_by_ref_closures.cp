// A closure capturing a `var` by reference sees and makes writes when it runs
// away from where it was made: from a fn it is passed to and from a std method.
fn apply(f: (Int) -> (), n: Int):
    for i in 0..n: f(i)
fn twice(f: () -> Int) -> Int = f() + f()
var c = 0
let f = (x: Int) => c += x
apply(f, 4)
print(c)
let n = [1, 2, 3].count(x =>
    c += x
    x > 1)
print(n, c)
var t = 5
print(twice(() =>
    t += 1
    t), t)
