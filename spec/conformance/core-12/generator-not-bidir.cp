// expect: E0606
// rule: 12.4.4
fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0: yield i

print(evens(10).last) // <- here
