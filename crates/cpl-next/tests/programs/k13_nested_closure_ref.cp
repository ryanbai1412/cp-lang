// expect: ok, output "22"
// A closure inside a closure, both sharing the outer var.
fn go() -> Int:
    var c = 0
    let outer = (k: Int) =>
        let inner = () => c += k
        inner()
        inner()
    outer(1)
    outer(10)
    c
print(go())
