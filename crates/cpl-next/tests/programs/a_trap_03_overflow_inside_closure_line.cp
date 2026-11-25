// expect: R0202
// rule: R09.3, trap at the line of the failing operation
// A trap is reported at the line where the failing operation is written, here inside a closure made on line 7 and called on line 9.
fn go(n: Int) -> Int:
    var x = n
    let f = () =>
        x = x * x  // <- here
        x * x
    let a = 1
    f() + a
print(go(3037000500))
