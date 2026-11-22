// expect: ok, output "60"
// rule: 13.2.13
fn f(k: Int) -> Int:
    domain V = 1..=k
    var inner = 0
    if k > 1:
        inner = f(k - 1)
    return V.count * 10 + inner
print(f(3))
