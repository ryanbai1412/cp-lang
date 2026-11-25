// expect: E0602
// A generator that takes `mut` and writes between yields; body reads the same list.
fn drain(mut xs: List<Int>) -> yields Int:
    var i = 0
    while i < xs.len:
        yield xs[i]
        xs.push(0 - 1)
        i += 2
fn go() -> Int:
    var xs: List<Int> = [1, 2, 3]
    var s = 0
    for v in drain(mut xs): s += v * 10 + xs.len  // <- here
    s
print(go())
