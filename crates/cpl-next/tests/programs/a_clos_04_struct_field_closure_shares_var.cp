// expect: ok, output "1 2"
// rule: R05.10.1, R05.10.2
// A struct holding a closure over a var, called twice through the field: shares the var.
struct Counter:
    next: () -> Int
fn go() -> Int:
    var c = 0
    let k = Counter(() => { c += 1; c })
    let a = k.next()
    let b = k.next()
    print(a, b)
    c
let r = go()
