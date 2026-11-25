// expect: ok, output "3"
// rule: R05.10.1
// A closure captured by another closure, both sharing one var two levels up.
fn go() -> Int:
    var c = 0
    let inc = () => c += 1
    let twice = () => { inc(); inc() }
    twice()
    inc()
    c
print(go())
