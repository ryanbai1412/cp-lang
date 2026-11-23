// `for x in f(xs)` over a `yields` fn with a list argument runs its body
// as `f` yields, in the C++ as in `cpl run`: what `f` prints after the
// value the body leaves on never happens (R13.3.3).
fn evens(xs: List<Int>) -> yields Int:
    for x in xs:
        if x % 2 == 0:
            print("yield", x)
            yield x
fn total(xs: List<Int>) -> Int:
    var t = 0
    for i in 0..3:
        for y in evens(xs):
            if y > 4: break
            t += y * i
    t
for y in evens([2, 4, 6, 8]):
    print("got", y)
    if y == 4: break
print(total([1, 2, 3, 4, 5, 6]))
