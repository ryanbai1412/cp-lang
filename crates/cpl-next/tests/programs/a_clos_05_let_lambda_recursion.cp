// expect: ok, output "1"
// rule: R05.10.5
// Recursion through a let-bound lambda (R05.10.5).
fn go(n: Int) -> Int:
    let fact = (k: Int) => if k <= 1: 1 else: k + fact(k - 1)
    fact(n) - 5 - 4
print(go(4))
