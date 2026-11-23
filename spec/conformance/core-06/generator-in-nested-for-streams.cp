// expect: ok
// rule: 06.3.5 13.3.3
// tool: build -o -
// lacks: "= ::prog::evens(xs)"
// A non-recursive generator in a `for` inlines; inside a loop it must not build a list per round (R13.3.3).
fn evens(xs: List<Int>) -> yields Int:
    for x in xs:
        if x % 2 == 0: yield x
fn total(xs: List<Int>) -> Int:
    var t = 0
    for i in 0..xs.len:
        for y in evens(xs): t += y * i
    t
print(total([1, 2, 3, 4]))
