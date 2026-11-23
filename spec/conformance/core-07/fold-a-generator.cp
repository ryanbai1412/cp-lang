// expect: ok, output "12 6 4"
// rule: 07.1.2
// A generator is a `Seq`, so every fold takes it.
fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0: yield i
print(evens(7).sum(), evens(7).max()!, evens(7).count())
