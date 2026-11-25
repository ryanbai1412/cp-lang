// expect: E0601
// rule: diagnostics E0601
// two(mut xs, mut xs) inside a lambda.
fn two(mut a: List<Int>, mut b: List<Int>):
    a.push(1)
    b.push(2)
fn go() -> Int:
    let f = () =>
        var xs = [0]
        two(mut xs, mut xs)  // <- here
        xs.len
    f()
print(go())
