// expect: ok, output "3 3"
// rule: R05.10.1
// A closure that captures a local list by reference sees later pushes (R05.10.1: local heap values are captured by reference).
fn go() -> Int:
    var xs = [1, 2]
    let n = () => xs.len
    xs.push(3)
    n()
print(go(), go())
