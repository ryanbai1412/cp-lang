// expect: ok, output "116"
// rule: 12.4.1
fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0: yield i

print(evens(10).map(x => x * x).filter(x => x > 10).sum())
