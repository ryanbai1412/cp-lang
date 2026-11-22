// `var (a, b) = ...` declares vars: each name is its own copy and can be assigned.
var (p, q) = (1, 2)
p += 5
q = p * 1000000
var xs: List<(Int, Int)> = [(1, 2)]
var (a, b) = xs[0]
a += 1
xs.push((a, b))
print(p, q, a, b, xs[0].0)
fn f() -> Int:
    var (lo, hi) = (0, 10)
    while lo < hi:
        lo += 1
        hi -= 1
    lo
print(f())
