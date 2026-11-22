// expect: ok, output "6"
// rule: 02.9.4
fn f(n: Int) -> Int:
    domain V = 0..n
    let inner = if n > 1: f(n - 1) else: 0
    V.count + inner
print(f(3))
