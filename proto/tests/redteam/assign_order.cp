var c = 0
fn nx() -> Int:
    c += 1
    return c
var a: [0..9]Int = 0
var xs: List<Int> = [5, 6, 7]
a[nx()] = nx()
a[nx()] += nx()
print(a)
let t = xs.pop()!
xs[0] = t
print(xs)
var x = 3
fn inc(mut v: Int) -> Int:
    v += 1
    return v
x = inc(mut x)
print(x, nx() + c, c + nx())
