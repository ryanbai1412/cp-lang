// expect: ok, output "44"
// A local heap `var` captured: by reference per R05.10.1. Push through the closure, read outside.
fn go() -> Int:
    var xs: List<Int> = []
    let add = (x: Int) => xs.push(x)
    add(1)
    add(2)
    xs.push(3)
    add(4)
    xs.len * 10 + xs[3]
print(go())
