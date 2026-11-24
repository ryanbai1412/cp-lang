// expect: ok, output "2"
// rule: 13.1.1 05.4.1
// An unused `let` still makes its call, so the callee's write through `mut` happens (adversarial round 46).
fn bump(mut c: Int) -> Int:
    c += 1
    return c
var n = 0
let unused = bump(mut n)
let again = bump(mut n)
print(n)
