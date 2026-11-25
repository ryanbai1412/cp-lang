// expect: E0601
// xs.extend(xs): one place lent mut and passed by value. Reject, or copy with a cost note.
fn app(mut to: List<Int>, from: List<Int>):
    for x in from: to.push(x)
fn go() -> Int:
    var xs: List<Int> = [1, 2, 3]
    app(mut xs, xs)  // <- here
    xs.len
print(go())
