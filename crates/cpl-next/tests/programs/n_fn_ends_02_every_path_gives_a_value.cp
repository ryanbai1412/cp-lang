// expect: ok, output "3 1 2 7"
// rule: R06.1.1
// A body ends with a value on every path when its last loop never ends (`while true` with no
// `break` out of it), every `if` has an `else` and every `match` arm gives one.
fn spin(x: Int) -> Int:
    while true:
        for i in 0..x:
            if i > 1: break
        if x > 0: return x
fn sign(x: Int) -> Int:
    if x > 0:
        1
    else:
        return 2
fn pick(x: Int) -> Int:
    match x:
        0 => 2
        _ => return 7
fn outer(k: Int) -> Int:
    fn inner(x: Int) -> Int:
        while true:
            if x > 0: return x
    inner(k)
print(spin(3), sign(1), pick(0), outer(7))
