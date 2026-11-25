// expect: ok, output "3"
// rule: R05.10.1
// R05.10.1: a `let` heap value is captured by reference too (not copied): cost only; value same.
fn go() -> Int:
    let xs = [1, 2, 3]
    let n = () => xs.len
    n()
print(go())
