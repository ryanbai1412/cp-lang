// expect: ok, output "6"
// rule: 06.3.1
fn evens(n: Int) -> yields Int:
    for i in 0..n if i % 2 == 0:
        yield i
print(evens(5).sum())
