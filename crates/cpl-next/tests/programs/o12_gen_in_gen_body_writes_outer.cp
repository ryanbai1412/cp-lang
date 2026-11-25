// expect: E0602
// Inner generator is passed the walked local as an argument; inner body writes it.
fn dbl(xs: List<Int>) -> yields Int:
    for x in xs:
        yield x
        yield x
fn go() -> Int:
    var xs: List<Int> = [1, 2]
    var n = 0
    for a in dbl(xs):
        n += a  // <- here
        if n < 30: xs.push(a)
    n * 100 + xs.len
print(go())
