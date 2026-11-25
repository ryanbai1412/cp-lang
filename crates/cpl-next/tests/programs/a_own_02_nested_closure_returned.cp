// expect: E0603
// rule: R05.7.8, R05.10.2
// R05.7.8: a lent value is not returned, also from a lambda: g returns a closure over its var.
fn make() -> () -> Int:
    var c = 0
    let outer = () => (() => { c += 1; c })  // <- here
    outer()
let f = make()
print(f(), f())
